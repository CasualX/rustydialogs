
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::*;



pub fn message_box(p: &MessageBox<'_>) -> Option<MessageResult> {
	let kind = match (p.buttons, p.icon) {
		(MessageButtons::Ok, MessageIcon::Info | MessageIcon::Question) => "--msgbox",
		(MessageButtons::Ok, MessageIcon::Warning) => "--sorry",
		(MessageButtons::Ok, MessageIcon::Error) => "--error",
		(MessageButtons::OkCancel, MessageIcon::Info | MessageIcon::Question) => "--yesno",
		(MessageButtons::OkCancel, MessageIcon::Warning | MessageIcon::Error) => "--warningyesno",
		(MessageButtons::YesNo, MessageIcon::Info | MessageIcon::Question) => "--yesno",
		(MessageButtons::YesNo, MessageIcon::Warning | MessageIcon::Error) => "--warningyesno",
		(MessageButtons::YesNoCancel, MessageIcon::Info | MessageIcon::Question) => "--yesnocancel",
		(MessageButtons::YesNoCancel, MessageIcon::Warning | MessageIcon::Error) => "--warningyesnocancel",
	};

	let (yes_label, no_label) = match p.buttons {
		MessageButtons::Ok => ("OK", "None"),
		MessageButtons::OkCancel => ("OK", "Cancel"),
		MessageButtons::YesNo => ("Yes", "No"),
		MessageButtons::YesNoCancel => ("Yes", "No"),
	};

	let codes: &[_] = match p.buttons {
		MessageButtons::Ok => &[MessageResult::Ok],
		MessageButtons::OkCancel => &[MessageResult::Ok, MessageResult::Cancel],
		MessageButtons::YesNo => &[MessageResult::Yes, MessageResult::No],
		MessageButtons::YesNoCancel => &[MessageResult::Yes, MessageResult::No, MessageResult::Cancel],
	};

	let status = invoke("kdialog", &[
		os(kind), os(p.message),
		os("--yes-label"), os(yes_label),
		os("--no-label"), os(no_label),
		os("--title"), os(p.title),
	]);

	let Some(index) = status else {
		exit_status_error(status);
	};
	codes.get(index as usize).copied()
}



pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	pick_files_impl(p, false).and_then(|paths| paths.into_iter().next())
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	pick_files_impl(p, true)
}

fn pick_files_impl(p: &FileDialog<'_>, multiple: bool) -> Option<Vec<PathBuf>> {
	let filter_string = filter_string(p.filter);
	let file_path = file_path(p.directory, p.file_name);
	let args = [
		os("--title"), os(p.title),
		os("--getopenfilename"), &file_path, os(&filter_string),
		os("--multiple"),
		os("--separate-output"),
	];
	let args = if multiple { &args[..] } else { &args[..args.len() - 2] };
	let (code, output) = invoke_output_bytes("kdialog", args);
	if code != Some(0) {
		return None;
	}

	Some(output.split(|&byte| byte == b'\n')
		.filter(|line| !line.is_empty())
		.map(|line| PathBuf::from(OsStr::from_bytes(line)))
		.collect::<Vec<_>>())
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let filter_string = filter_string(p.filter);
	let file_path = file_path(p.directory, p.file_name);

	let args = [
		os("--title"), os(p.title),
		os("--getsavefilename"), &file_path, os(&filter_string),
	];

	let (code, output) = invoke_output_bytes("kdialog", &args);
	if code != Some(0) {
		return None;
	}

	output
		.split(|&b| b == b'\n')
		.find(|line| !line.is_empty())
		.map(|line| PathBuf::from(OsStr::from_bytes(line)))
}

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	let directory = p.directory.unwrap_or_else(|| Path::new("."));
	let args = [
		os("--title"), os(p.title),
		os("--getexistingdirectory"), directory.as_os_str(),
	];

	let (code, output) = invoke_output_bytes("kdialog", &args);
	if code != Some(0) {
		return None;
	}

	output
		.split(|&b| b == b'\n')
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

fn filter_string(filter: Option<&[FileFilter<'_>]>) -> String {
	let mut result = String::new();
	if let Some(filter) = filter {
		for entry in filter {
			_ = write!(result, "{} ({})\n", entry.desc, utils::PrintJoin { parts: entry.patterns, separator: " " });
		}
	}
	result.push_str("All Files (*)");
	result
}



pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	let args: &[&OsStr] = match p.mode {
		TextInputMode::Single => &[os("--inputbox"), os(p.message), os(p.value), os("--title"), os(p.title)],
		TextInputMode::Multi => &[os("--textinputbox"), os(p.message), os(p.value), os("--title"), os(p.title)],
		TextInputMode::Password => &[os("--password"), os(p.message), os("--title"), os(p.title)],
	};
	let (status, output) = invoke_output("kdialog", args);

	match status {
		Some(0) => Some(output),
		_ => None,
	}
}



pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	let value = format_color(p.value);
	let (status, output) = invoke_output("kdialog", &[os("--getcolor"), os(&value), os("--title"), os(p.title)]);

	if status != Some(0) {
		return None;
	}

	let color = parse_color(&output).unwrap_or_else(|| panic!("kdialog color_picker returned invalid color: {output}"));
	Some(color)
}

fn format_color(color: ColorValue) -> String {
	format!("#{:02X}{:02X}{:02X}", color.red, color.green, color.blue)
}

fn parse_color(value: &str) -> Option<ColorValue> {
	let value = value.trim();
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

	let timeout_seconds = timeout_seconds(p.timeout).to_string();

	let args = &[
		os("--title"), os(p.title),
		os("--icon"), os(icon),
		os("--passivepopup"), os(p.message),
		os(&timeout_seconds),
	];

	invoke_async("kdialog", args);
}

fn timeout_seconds(timeout: i32) -> u64 {
	if timeout <= 0 {
		// KDialog does not support infinite timeouts, so we use the maximum possible timeout instead.
		i32::MAX as u64
	}
	else {
		(timeout as u32).div_ceil(1000) as u64
	}
}
