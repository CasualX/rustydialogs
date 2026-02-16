use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use super::*;



pub fn message_box(p: &MessageBox<'_>) -> Option<MessageResult> {
	let mut args = vec![
		os("--title"),
		os(p.title),
		os("--text"),
		os(p.message),
	];

	match p.icon {
		MessageIcon::Info => args.push(os("--info")),
		MessageIcon::Warning => args.push(os("--warning")),
		MessageIcon::Error => args.push(os("--error")),
		MessageIcon::Question => args.push(os("--question")),
	}

	match p.buttons {
		MessageButtons::Ok => {
			args.push(os("--ok-label"));
			args.push(os("OK"));
		}
		MessageButtons::OkCancel => {
			args.push(os("--ok-label"));
			args.push(os("OK"));
			args.push(os("--extra-button"));
			args.push(os("Cancel"));
		}
		MessageButtons::YesNo => {
			args.push(os("--ok-label"));
			args.push(os("Yes"));
			args.push(os("--extra-button"));
			args.push(os("No"));
		}
		MessageButtons::YesNoCancel => {
			args.push(os("--ok-label"));
			args.push(os("Yes"));
			args.push(os("--extra-button"));
			args.push(os("Cancel"));
			args.push(os("--extra-button"));
			args.push(os("No"));
		}
	}

	let (status, output) = invoke_output("zenity", &args);
	if !(status == Some(0) || status == Some(1)) {
		exit_status_error(status);
	}

	match output.trim() {
		"OK" => Some(MessageResult::Ok),
		"Cancel" => Some(MessageResult::Cancel),
		"Yes" => Some(MessageResult::Yes),
		"No" => Some(MessageResult::No),
		_ => Some(MessageResult::Ok), // Default to Ok for unknown output
	}
}



pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	pick_files_impl(p, false).and_then(|paths| paths.into_iter().next())
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	pick_files_impl(p, true)
}

fn pick_files_impl(p: &FileDialog<'_>, multiple: bool) -> Option<Vec<PathBuf>> {
	let file_path = file_path(p.directory, p.file_name);
	let mut args = vec![
		os("--file-selection"),
		os("--title"),
		os(p.title),
		os("--filename"),
		&file_path,
	];

	if multiple {
		args.push(os("--multiple"));
		args.push(os("--separator"));
		args.push(os("\n"));
	}

	let filters = filter_strings(p.filter);
	for filter in &filters {
		args.push(os("--file-filter"));
		args.push(os(filter));
	}

	let (code, output) = invoke_output_bytes("zenity", &args);
	if code != Some(0) {
		return None;
	}

	Some(output.split(|&byte| byte == b'\n')
		.filter(|line| !line.is_empty())
		.map(|line| PathBuf::from(OsStr::from_bytes(line)))
		.collect::<Vec<_>>())
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let file_path = file_path(p.directory, p.file_name);
	let mut args = vec![
		os("--file-selection"),
		os("--save"),
		os("--confirm-overwrite"),
		os("--title"),
		os(p.title),
		os("--filename"),
		&file_path,
	];

	let filters = filter_strings(p.filter);
	for filter in &filters {
		args.push(os("--file-filter"));
		args.push(os(filter));
	}

	let (code, output) = invoke_output_bytes("zenity", &args);
	if code != Some(0) {
		return None;
	}

	output
		.split(|&byte| byte == b'\n')
		.find(|line| !line.is_empty())
		.map(|line| PathBuf::from(OsStr::from_bytes(line)))
}

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	let directory = p.directory.unwrap_or_else(|| Path::new("."));
	let args = [
		os("--file-selection"),
		os("--directory"),
		os("--title"),
		os(p.title),
		os("--filename"),
		directory.as_os_str(),
	];

	let (code, output) = invoke_output_bytes("zenity", &args);
	if code != Some(0) {
		return None;
	}

	output
		.split(|&byte| byte == b'\n')
		.find(|line| !line.is_empty())
		.map(|line| PathBuf::from(OsStr::from_bytes(line)))
}

fn file_path(directory: Option<&Path>, file_name: Option<&Path>) -> OsString {
	let path = match (directory, file_name) {
		(Some(dir), Some(file)) => dir.join(file),
		(Some(dir), None) => dir.to_path_buf(),
		(None, Some(file)) => file.to_path_buf(),
		(None, None) => PathBuf::from("."),
	};
	path.into_os_string()
}

fn filter_strings(filter: Option<&[FileFilter<'_>]>) -> Vec<String> {
	let mut result = Vec::new();
	if let Some(filter) = filter {
		for entry in filter {
			result.push(format!("{} | {}", entry.desc, utils::PrintJoin { parts: entry.patterns, separator: " " }));
		}
	}
	result.push(String::from("All Files (*) | *"));
	result
}



pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	match p.mode {
		TextInputMode::Single => text_input_single(p),
		TextInputMode::Multi => text_input_multi(p),
		TextInputMode::Password => text_input_password(p),
	}
}

fn text_input_single(p: &TextInput<'_>) -> Option<String> {
	let args = [
		os("--entry"),
		os("--title"),
		os(p.title),
		os("--text"),
		os(p.message),
		os("--entry-text"),
		os(p.value),
	];
	let (status, output) = invoke_output("zenity", &args);
	if status == Some(0) { Some(output) } else { None }
}

fn text_input_multi(p: &TextInput<'_>) -> Option<String> {
	let temp_path = temp_file_path("rustydialogs-zenity-multi");
	if std::fs::write(&temp_path, p.value.as_bytes()).is_err() {
		return None;
	}

	let args = [
		os("--text-info"),
		os("--editable"),
		os("--title"),
		os(p.title),
		os("--filename"),
		temp_path.as_os_str(),
	];
	let (status, output) = invoke_output("zenity", &args);
	let _ = std::fs::remove_file(&temp_path);
	if status == Some(0) { Some(output) } else { None }
}

fn text_input_password(p: &TextInput<'_>) -> Option<String> {
	let args = [
		os("--password"),
		os("--title"),
		os(p.title),
		os("--text"),
		os(p.message),
	];
	let (status, output) = invoke_output("zenity", &args);
	if status == Some(0) { Some(output) } else { None }
}

fn temp_file_path(prefix: &str) -> PathBuf {
	let nanos = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map_or(0, |d| d.as_nanos());
	std::env::temp_dir().join(format!("{prefix}-{}-{nanos}.txt", process::id()))
}



pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	let value = format_color(p.value);
	let args = [
		os("--color-selection"),
		os("--title"),
		os(p.title),
		os("--color"),
		os(&value),
	];
	let (status, output) = invoke_output("zenity", &args);

	if status != Some(0) {
		return None;
	}

	let color = parse_color(&output)
		.unwrap_or_else(|| panic!("zenity color_picker returned invalid color: {output}"));
	Some(color)
}

fn format_color(color: ColorValue) -> String {
	format!("#{:02X}{:02X}{:02X}", color.red, color.green, color.blue)
}

fn parse_color(value: &str) -> Option<ColorValue> {
	let value = value.trim();

	if let Some(value) = value.strip_prefix("rgb(").and_then(|v| v.strip_suffix(')')) {
		let mut parts = value.split(',').map(str::trim);
		let red = parts.next()?.parse::<u8>().ok()?;
		let green = parts.next()?.parse::<u8>().ok()?;
		let blue = parts.next()?.parse::<u8>().ok()?;
		if parts.next().is_some() {
			return None;
		}
		return Some(ColorValue { red, green, blue });
	}

	let value = value.strip_prefix('#').unwrap_or(value);
	if value.len() != 6 && value.len() != 8 {
		return None;
	}
	if !value.is_ascii() {
		return None;
	}

	let red = u8::from_str_radix(&value[0..2], 16).ok()?;
	let green = u8::from_str_radix(&value[2..4], 16).ok()?;
	let blue = u8::from_str_radix(&value[4..6], 16).ok()?;
	if value.len() == 8 {
		u8::from_str_radix(&value[6..8], 16).ok()?;
	}

	Some(ColorValue { red, green, blue })
}



pub fn notify_popup(p: &NotifyPopup<'_>) {
	let icon = match p.icon {
		MessageIcon::Info | MessageIcon::Question => "dialog-information",
		MessageIcon::Warning => "dialog-warning",
		MessageIcon::Error => "dialog-error",
	};

	let text = format!("{}\n{}", p.title, p.message);

	let mut args = vec![
		os("--notification"),
		os("--icon"),
		os(icon),
		os("--text"),
		os(&text),
	];

	let timeout_storage;
	if let Some(timeout_seconds) = timeout_seconds(p.timeout) {
		timeout_storage = format!("--timeout={timeout_seconds}");
		args.push(os(&timeout_storage));
	}

	invoke_async("zenity", &args);
}

fn timeout_seconds(timeout_milliseconds: i32) -> Option<u64> {
	if timeout_milliseconds <= 0 {
		return None;
	}
	Some((timeout_milliseconds as u32).div_ceil(1000) as u64)
}
