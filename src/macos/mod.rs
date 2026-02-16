use std::path::{Path, PathBuf};
use std::process;

use super::*;

pub fn show(p: &MessageBox<'_>) -> Option<MessageResult> {
	let icon = match p.icon {
		MessageIcon::Info | MessageIcon::Question => "note",
		MessageIcon::Warning => "caution",
		MessageIcon::Error => "stop",
	};

	let (buttons, default_button, cancel_button) = match p.buttons {
		MessageButtons::Ok => ("OK", "OK", ""),
		MessageButtons::OkCancel => ("OK||Cancel", "OK", "Cancel"),
		MessageButtons::YesNo => ("Yes||No", "Yes", ""),
		MessageButtons::YesNoCancel => ("Yes||No||Cancel", "Yes", "Cancel"),
	};

	let script = r#"
on run argv
	set theTitle to item 1 of argv
	set theMessage to item 2 of argv
	set theIcon to item 3 of argv
	set theButtonsText to item 4 of argv
	set theDefaultButton to item 5 of argv
	set theCancelButton to item 6 of argv

	set oldDelims to AppleScript's text item delimiters
	set AppleScript's text item delimiters to "||"
	set buttonList to every text item of theButtonsText
	set AppleScript's text item delimiters to oldDelims

	set iconSpec to note
	if theIcon is "caution" then set iconSpec to caution
	if theIcon is "stop" then set iconSpec to stop

	if theCancelButton is "" then
		set response to display dialog theMessage with title theTitle buttons buttonList default button theDefaultButton with icon iconSpec
	else
		set response to display dialog theMessage with title theTitle buttons buttonList default button theDefaultButton cancel button theCancelButton with icon iconSpec
	end if

	return button returned of response
end run
"#;

	let output = invoke_output(script, &[p.title, p.message, icon, buttons, default_button, cancel_button])?;

	match output.as_str() {
		"OK" => Some(MessageResult::Ok),
		"Cancel" => Some(MessageResult::Cancel),
		"Yes" => Some(MessageResult::Yes),
		"No" => Some(MessageResult::No),
		_ => None,
	}
}

pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	pick_files_impl(p, false).and_then(|files| files.into_iter().next())
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	pick_files_impl(p, true)
}

fn pick_files_impl(p: &FileDialog<'_>, multiple: bool) -> Option<Vec<PathBuf>> {
	let initial_directory = initial_directory(p.directory, p.file_name)
		.map(|path| path.to_string_lossy().into_owned())
		.unwrap_or_default();

	let script_single = r#"
on run argv
	set theTitle to item 1 of argv
	set initialPath to item 2 of argv

	if initialPath is "" then
		set selectedFile to choose file with prompt theTitle
	else
		set selectedFile to choose file with prompt theTitle default location (POSIX file initialPath)
	end if

	return POSIX path of selectedFile
end run
"#;

	let script_multi = r#"
on run argv
	set theTitle to item 1 of argv
	set initialPath to item 2 of argv

	if initialPath is "" then
		set selectedFiles to choose file with prompt theTitle with multiple selections allowed true
	else
		set selectedFiles to choose file with prompt theTitle default location (POSIX file initialPath) with multiple selections allowed true
	end if

	set outputLines to {}
	repeat with selectedFile in selectedFiles
		set end of outputLines to POSIX path of selectedFile
	end repeat

	set oldDelims to AppleScript's text item delimiters
	set AppleScript's text item delimiters to linefeed
	set joined to outputLines as text
	set AppleScript's text item delimiters to oldDelims
	return joined
end run
"#;

	let script = if multiple { script_multi } else { script_single };
	let output = invoke_output(script, &[p.title, &initial_directory])?;

	let files = output
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty())
		.map(PathBuf::from)
		.collect::<Vec<_>>();

	if files.is_empty() {
		None
	}
	else {
		Some(files)
	}
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let initial_directory = initial_directory(p.directory, p.file_name)
		.map(|path| path.to_string_lossy().into_owned())
		.unwrap_or_default();
	let default_name = p.file_name
		.and_then(|name| name.file_name())
		.and_then(|name| name.to_str())
		.unwrap_or("");

	let script = r#"
on run argv
	set theTitle to item 1 of argv
	set initialPath to item 2 of argv
	set defaultName to item 3 of argv

	if initialPath is "" then
		if defaultName is "" then
			set savedFile to choose file name with prompt theTitle
		else
			set savedFile to choose file name with prompt theTitle default name defaultName
		end if
	else
		if defaultName is "" then
			set savedFile to choose file name with prompt theTitle default location (POSIX file initialPath)
		else
			set savedFile to choose file name with prompt theTitle default location (POSIX file initialPath) default name defaultName
		end if
	end if

	return POSIX path of savedFile
end run
"#;

	let output = invoke_output(script, &[p.title, &initial_directory, default_name])?;
	let path = output.trim();

	if path.is_empty() {
		None
	}
	else {
		Some(PathBuf::from(path))
	}
}

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	let initial_directory = p.directory
		.and_then(|path| path.to_str())
		.unwrap_or("");

	let script = r#"
on run argv
	set theTitle to item 1 of argv
	set initialPath to item 2 of argv

	if initialPath is "" then
		set selectedFolder to choose folder with prompt theTitle
	else
		set selectedFolder to choose folder with prompt theTitle default location (POSIX file initialPath)
	end if

	return POSIX path of selectedFolder
end run
"#;

	let output = invoke_output(script, &[p.title, initial_directory])?;
	let path = output.trim();

	if path.is_empty() {
		None
	}
	else {
		Some(PathBuf::from(path))
	}
}

pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	let hidden = if p.mode == TextInputMode::Password { "true" } else { "false" };

	let script = r#"
on run argv
	set theTitle to item 1 of argv
	set theMessage to item 2 of argv
	set theValue to item 3 of argv
	set hiddenValue to item 4 of argv

	if hiddenValue is "true" then
		set response to display dialog theMessage with title theTitle default answer theValue with hidden answer
	else
		set response to display dialog theMessage with title theTitle default answer theValue
	end if

	return text returned of response
end run
"#;

	invoke_output(script, &[p.title, p.message, p.value, hidden])
}

pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	let red = ((p.value.red as u16) * 257).to_string();
	let green = ((p.value.green as u16) * 257).to_string();
	let blue = ((p.value.blue as u16) * 257).to_string();

	let script = r#"
on run argv
	set theTitle to item 1 of argv
	set redValue to (item 2 of argv) as integer
	set greenValue to (item 3 of argv) as integer
	set blueValue to (item 4 of argv) as integer

	set selectedColor to choose color default color {redValue, greenValue, blueValue}
	set r to item 1 of selectedColor
	set g to item 2 of selectedColor
	set b to item 3 of selectedColor
	return (r as string) & "," & (g as string) & "," & (b as string)
end run
"#;

	let output = invoke_output(script, &[p.title, &red, &green, &blue])?;
	parse_color(&output)
}

pub fn notify_popup(p: &NotifyPopup<'_>) {
	let script = r#"
on run argv
	set theTitle to item 1 of argv
	set theMessage to item 2 of argv
	display notification theMessage with title theTitle
end run
"#;

	invoke_async(script, &[p.title, p.message]);
}

#[track_caller]
fn invoke_output(script: &str, args: &[&str]) -> Option<String> {
	let output = process::Command::new("osascript")
		.arg("-e")
		.arg(script)
		.args(args)
		.output()
		.unwrap();

	if !output.status.success() {
		return None;
	}

	let mut stdout = String::from_utf8(output.stdout)
		.unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).to_string());
	while matches!(stdout.chars().last(), Some('\n' | '\r')) {
		stdout.pop();
	}
	Some(stdout)
}

#[track_caller]
fn invoke_async(script: &str, args: &[&str]) {
	let _ = process::Command::new("osascript")
		.arg("-e")
		.arg(script)
		.args(args)
		.spawn()
		.unwrap();
}

fn initial_directory(directory: Option<&Path>, file_name: Option<&Path>) -> Option<PathBuf> {
	if let Some(directory) = directory {
		return Some(directory.to_path_buf());
	}

	file_name.and_then(Path::parent).map(Path::to_path_buf)
}

fn parse_color(value: &str) -> Option<ColorValue> {
	let mut parts = value.trim().split(',').map(str::trim);
	let red16 = parts.next()?.parse::<u16>().ok()?;
	let green16 = parts.next()?.parse::<u16>().ok()?;
	let blue16 = parts.next()?.parse::<u16>().ok()?;

	if parts.next().is_some() {
		return None;
	}

	Some(ColorValue {
		red: (red16 / 257) as u8,
		green: (green16 / 257) as u8,
		blue: (blue16 / 257) as u8,
	})
}
