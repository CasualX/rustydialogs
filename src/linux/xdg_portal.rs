use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::{thread, time};

use dbus::arg::{PropMap, RefArg, Variant};
use dbus::blocking::Connection;
use dbus::message::MatchRule;

use super::*;

const DESKTOP_BUS_NAME: &str = "org.freedesktop.portal.Desktop";
const DESKTOP_PATH: &str = "/org/freedesktop/portal/desktop";
const FILE_CHOOSER_INTERFACE: &str = "org.freedesktop.portal.FileChooser";
const NOTIFICATION_INTERFACE: &str = "org.freedesktop.portal.Notification";
const REQUEST_INTERFACE: &str = "org.freedesktop.portal.Request";

static NEXT_NOTIFICATION_ID: AtomicU64 = AtomicU64::new(1);

pub fn message_box(_: &MessageBox<'_>) -> Option<MessageResult> {
	None
}

pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	pick_files_impl(p, false).and_then(|paths| paths.into_iter().next())
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	pick_files_impl(p, true)
}

fn pick_files_impl(p: &FileDialog<'_>, multiple: bool) -> Option<Vec<PathBuf>> {
	let conn = Connection::new_session().ok()?;
	let proxy = conn.with_proxy(DESKTOP_BUS_NAME, DESKTOP_PATH, time::Duration::from_secs(30));

	let mut options = portal_file_options(p);
	options.insert(String::from("multiple"), Variant(Box::new(multiple)));

	let (request_path,): (dbus::Path<'static>,) = proxy
		.method_call(FILE_CHOOSER_INTERFACE, "OpenFile", (String::new(), p.title, options))
		.ok()?;

	let (response, results) = wait_portal_response(&conn, request_path, time::Duration::from_secs(120))?;
	if response != 0 {
		return None;
	}
	let uris = result_uris(&results)?;

	let paths = uris
		.into_iter()
		.filter_map(|uri| parse_file_uri(&uri))
		.collect::<Vec<_>>();

	if paths.is_empty() {
		None
	}
	else {
		Some(paths)
	}
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let conn = Connection::new_session().ok()?;
	let proxy = conn.with_proxy(DESKTOP_BUS_NAME, DESKTOP_PATH, time::Duration::from_secs(30));

	let options = portal_file_options(p);

	let (request_path,): (dbus::Path<'static>,) = proxy
		.method_call(FILE_CHOOSER_INTERFACE, "SaveFile", (String::new(), p.title, options))
		.ok()?;

	let (response, results) = wait_portal_response(&conn, request_path, time::Duration::from_secs(120))?;
	if response != 0 {
		return None;
	}
	let uris = result_uris(&results)?;
	uris.into_iter().find_map(|uri| parse_file_uri(&uri))
}

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	let conn = Connection::new_session().ok()?;
	let proxy = conn.with_proxy(DESKTOP_BUS_NAME, DESKTOP_PATH, time::Duration::from_secs(30));

	let mut options: PropMap = PropMap::new();
	options.insert(String::from("directory"), Variant(Box::new(true)));
	if let Some(directory) = p.directory {
		if let Some(folder) = portal_directory_bytes(directory) {
			options.insert(String::from("current_folder"), Variant(Box::new(folder)));
		}
	}

	let (request_path,): (dbus::Path<'static>,) = proxy
		.method_call(FILE_CHOOSER_INTERFACE, "OpenFile", (String::new(), p.title, options))
		.ok()?;

	let (response, results) = wait_portal_response(&conn, request_path, time::Duration::from_secs(120))?;
	if response != 0 {
		return None;
	}
	let uris = result_uris(&results)?;
	uris.into_iter().find_map(|uri| parse_file_uri(&uri))
}

fn portal_file_options(p: &FileDialog<'_>) -> PropMap {
	let mut options: PropMap = PropMap::new();
	let path = utils::abspath(p.path);
	if path.is_dir() {
		if let Some(folder) = portal_directory_bytes(&path) {
			options.insert(String::from("current_folder"), Variant(Box::new(folder)));
		}
	}
	else {
		if let Some(parent) = path.parent() {
			if let Some(folder) = portal_directory_bytes(parent) {
				options.insert(String::from("current_folder"), Variant(Box::new(folder)));
			}
		}
		if let Some(name) = path.file_name() {
			options.insert(String::from("current_name"), Variant(Box::new(name.to_string_lossy().to_string())));
		}
	}
	let filters = portal_filters(p.filter);
	if !filters.is_empty() {
		options.insert(String::from("filters"), Variant(Box::new(filters.clone())));
		options.insert(String::from("current_filter"), Variant(Box::new(filters[0].clone())));
	}
	options
}

type PortalFilter = (String, Vec<(u32, String)>);

fn portal_filters(filters: Option<&[FileFilter<'_>]>) -> Vec<PortalFilter> {
	let mut out = filters
		.unwrap_or(&[])
		.iter()
		.map(|entry| {
			let patterns = entry
				.patterns
				.iter()
				.map(|pattern| (0u32, (*pattern).to_string()))
				.collect::<Vec<_>>();
			(entry.desc.to_string(), patterns)
		})
		.filter(|(_, patterns)| !patterns.is_empty())
		.collect::<Vec<_>>();

	out.push((String::from("All Files (*)"), vec![(0u32, String::from("*"))]));
	out
}

fn portal_directory_bytes(path: &Path) -> Option<Vec<u8>> {
	let absolute = if path.is_absolute() {
		path.to_path_buf()
	}
	else {
		std::env::current_dir().ok()?.join(path)
	};

	if !absolute.is_dir() {
		return None;
	}

	let mut bytes = absolute.as_os_str().as_bytes().to_vec();
	bytes.push(0);
	Some(bytes)
}

fn wait_portal_response(
	conn: &Connection,
	path: dbus::Path<'static>,
	timeout: time::Duration,
) -> Option<(u32, HashMap<String, Variant<Box<dyn RefArg + 'static>>>)> {
	let mut rule = MatchRule::new_signal(REQUEST_INTERFACE, "Response");
	rule.path = Some(path);

	let (tx, rx) = mpsc::channel();
	let _token = conn
		.add_match(rule, move |(response, results): (u32, HashMap<String, Variant<Box<dyn RefArg + 'static>>>), _, _| {
			let _ = tx.send((response, results));
			true
		})
		.ok()?;

	let deadline = time::Instant::now() + timeout;
	loop {
		if let Ok(result) = rx.try_recv() {
			return Some(result);
		}

		if time::Instant::now() >= deadline {
			return None;
		}

		if conn.process(time::Duration::from_millis(200)).is_err() {
			return None;
		}
	}
}

fn result_uris(results: &HashMap<String, Variant<Box<dyn RefArg + 'static>>>) -> Option<Vec<String>> {
	let uris = results.get("uris")?;
	let values = uris.0.as_iter()?;
	let mut out = Vec::new();
	for value in values {
		if let Some(uri) = value.as_str() {
			out.push(uri.to_string());
		}
	}
	if out.is_empty() {
		None
	}
	else {
		Some(out)
	}
}

fn parse_file_uri(uri: &str) -> Option<PathBuf> {
	let value = uri.strip_prefix("file://")?;
	let decoded = percent_decode(value)?;
	Some(PathBuf::from(decoded))
}

fn percent_decode(input: &str) -> Option<String> {
	let bytes = input.as_bytes();
	let mut out = Vec::with_capacity(bytes.len());
	let mut index = 0;
	while index < bytes.len() {
		let byte = bytes[index];
		if byte == b'%' {
			if index + 2 >= bytes.len() {
				return None;
			}
			let hi = hex_digit(bytes[index + 1])?;
			let lo = hex_digit(bytes[index + 2])?;
			out.push((hi << 4) | lo);
			index += 3;
		}
		else {
			out.push(byte);
			index += 1;
		}
	}
	String::from_utf8(out).ok()
}

fn hex_digit(digit: u8) -> Option<u8> {
	match digit {
		b'0'..=b'9' => Some(digit - b'0'),
		b'a'..=b'f' => Some(digit - b'a' + 10),
		b'A'..=b'F' => Some(digit - b'A' + 10),
		_ => None,
	}
}

pub fn text_input(_: &TextInput<'_>) -> Option<String> {
	None
}

pub fn color_picker(_: &ColorPicker<'_>) -> Option<ColorValue> {
	None
}

pub fn notify(p: &Notification<'_>) {
	let conn = match Connection::new_session() {
		Ok(conn) => conn,
		Err(_) => return,
	};
	let proxy = conn.with_proxy(DESKTOP_BUS_NAME, DESKTOP_PATH, time::Duration::from_secs(5));

	let app_id = p.app_id;
	let notification_id = format!("rustydialogs-{}", NEXT_NOTIFICATION_ID.fetch_add(1, Ordering::Relaxed));

	let mut notification: PropMap = PropMap::new();
	notification.insert(String::from("title"), Variant(Box::new(p.title.to_string())));
	notification.insert(String::from("body"), Variant(Box::new(p.message.to_string())));
	notification.insert(String::from("priority"), Variant(Box::new(notification_priority(p.icon).to_string())));

	let result: Result<(), _> = proxy.method_call(
		NOTIFICATION_INTERFACE,
		"AddNotification",
		(app_id, notification_id.as_str(), notification),
	);
	if result.is_err() {
		return;
	}

	if p.timeout > 0 {
		let timeout = p.timeout as u64;
		let app_id = app_id.to_string();
		thread::spawn(move || {
			thread::sleep(time::Duration::from_millis(timeout));
			let conn = match Connection::new_session() {
				Ok(conn) => conn,
				Err(_) => return,
			};
			let proxy = conn.with_proxy(DESKTOP_BUS_NAME, DESKTOP_PATH, time::Duration::from_secs(5));
			let _: Result<(), _> = proxy.method_call(
				NOTIFICATION_INTERFACE,
				"RemoveNotification",
				(app_id.as_str(), notification_id.as_str()),
			);
		});
	}
}

fn notification_priority(icon: MessageIcon) -> &'static str {
	match icon {
		MessageIcon::Info | MessageIcon::Question => "normal",
		MessageIcon::Warning => "high",
		MessageIcon::Error => "urgent",
	}
}
