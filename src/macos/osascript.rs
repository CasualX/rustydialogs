use std::process;

use super::*;

pub fn message_box(p: &MessageBox<'_>) -> Option<MessageResult> {
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
		try
			set response to display dialog theMessage with title theTitle buttons buttonList default button theDefaultButton cancel button theCancelButton with icon iconSpec
		on error number -128
			return theCancelButton
		end try
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
	let initial_directory = initial_directory(p.path)
		.map(|path| path.to_string_lossy().into_owned())
		.unwrap_or_default();

	let script = r#"
use framework "Foundation"
use scripting additions

on run argv
	set theTitle to item 1 of argv
	set initialPath to item 2 of argv
	set allowMultiple to (item 3 of argv is "true")
	set fileTypes to {}
	if (count argv) > 3 then
		try
			-- Four-character extensions otherwise get interpreted as legacy file type codes.
			set frameworkBundle to current application's NSBundle's bundleWithPath:"/System/Library/Frameworks/UniformTypeIdentifiers.framework"
			if frameworkBundle is missing value then error "Type identifiers unavailable"
			if not (frameworkBundle's load() as boolean) then error "Type identifiers unavailable"
			repeat with extensionValue in items 4 thru -1 of argv
				set fileType to current application's UTType's typeWithFilenameExtension:(extensionValue as text)
				set end of fileTypes to fileType's identifier() as text
			end repeat
		on error
			-- Older systems without UTType still get a usable, unrestricted picker.
			set fileTypes to {}
		end try
	end if

	if initialPath is "" then
		if fileTypes is {} then
			set selectedFiles to choose file with prompt theTitle multiple selections allowed allowMultiple
		else
			set selectedFiles to choose file with prompt theTitle of type fileTypes multiple selections allowed allowMultiple
		end if
	else
		if fileTypes is {} then
			set selectedFiles to choose file with prompt theTitle default location (POSIX file initialPath as alias) multiple selections allowed allowMultiple
		else
			set selectedFiles to choose file with prompt theTitle of type fileTypes default location (POSIX file initialPath as alias) multiple selections allowed allowMultiple
		end if
	end if
	if not allowMultiple then set selectedFiles to {selectedFiles}

	set outputLines to {}
	repeat with selectedFile in selectedFiles
		set end of outputLines to POSIX path of (contents of selectedFile)
	end repeat

	set oldDelims to AppleScript's text item delimiters
	set AppleScript's text item delimiters to character id 0
	set joined to outputLines as text
	set AppleScript's text item delimiters to oldDelims
	return joined
end run
"#;

	let file_types = filter_types(p.filters).unwrap_or_default();
	let mut args = vec![p.title, &initial_directory, if multiple { "true" } else { "false" }];
	args.extend(file_types.iter().map(String::as_str));
	let output = invoke_output(script, &args)?;
	parse_paths(&output)
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let (initial_directory, default_name) = initial_directory_and_name(p.path);
	let initial_directory = initial_directory
		.map(|path| path.to_string_lossy().into_owned())
		.unwrap_or_default();
	let default_name = default_name.as_deref().unwrap_or("");

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

	let path = invoke_output(script, &[p.title, &initial_directory, default_name])?;
	if path.is_empty() {
		return None;
	}
	Some(PathBuf::from(path))
}

pub fn choose_folder(p: &FileDialog<'_>) -> Option<PathBuf> {
	let initial_directory = p.path
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

	let path = invoke_output(script, &[p.title, initial_directory])?;
	if path.is_empty() {
		return None;
	}
	Some(PathBuf::from(path))
}

pub fn choose_folders(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	let initial_directory = p.path
		.and_then(|path| path.to_str())
		.unwrap_or("");

	let script = r#"
on run argv
	set theTitle to item 1 of argv
	set initialPath to item 2 of argv

	if initialPath is "" then
		set selectedFolders to choose folder with prompt theTitle with multiple selections allowed
	else
		set selectedFolders to choose folder with prompt theTitle default location (POSIX file initialPath) with multiple selections allowed
	end if

	set outputLines to {}
	repeat with selectedFolder in selectedFolders
		set end of outputLines to POSIX path of (contents of selectedFolder)
	end repeat

	set oldDelims to AppleScript's text item delimiters
	set AppleScript's text item delimiters to character id 0
	set joined to outputLines as text
	set AppleScript's text item delimiters to oldDelims
	return joined
end run
"#;

	let output = invoke_output(script, &[p.title, initial_directory])?;
	parse_paths(&output)
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

#[inline]
pub fn notify_setup(_app_id: &str) -> bool {
	// Nothing here yet...
	true
}

pub fn notify(p: &Notification<'_>) {
	if !notify_setup(p.app_id) {
		return;
	}

	let script = r#"
on run argv
	set theTitle to item 1 of argv
	set theMessage to item 2 of argv
	set theAppId to item 3 of argv
	display notification theMessage with title theTitle subtitle theAppId
end run
"#;

	invoke_async(script, &[p.title, p.message, p.app_id]);
}

#[track_caller]
fn invoke_output(script: &str, args: &[&str]) -> Option<String> {
	let output = process::Command::new("osascript").arg("-e").arg(script).arg("--").args(args).output().expect("failed to execute osascript");

	if !output.status.success() {
		return None;
	}

	let mut stdout = String::from_utf8(output.stdout).expect("failed to parse stdout as UTF-8");
	// osascript appends one linefeed; any other whitespace belongs to the result.
	if stdout.ends_with('\n') {
		stdout.pop();
	}
	Some(stdout)
}

#[track_caller]
fn invoke_async(script: &str, args: &[&str]) {
	let mut child = process::Command::new("osascript").arg("-e").arg(script).arg("--").args(args).spawn().expect("failed to execute osascript");
	// Reap the helper without blocking the caller.
	let _ = std::thread::spawn(move || { let _ = child.wait(); });
}

fn parse_paths(output: &str) -> Option<Vec<PathBuf>> {
	// NUL cannot occur in a filesystem path; line breaks can.
	let paths = output.split('\0').filter(|path| !path.is_empty()).map(PathBuf::from).collect::<Vec<_>>();
	if paths.is_empty() { None } else { Some(paths) }
}

fn filter_types(filters: Option<&[FileFilter<'_>]>) -> Option<Vec<String>> {
	let mut result = Vec::new();
	for filter in filters? {
		for pattern in filter.patterns {
			// Standard Additions accepts extensions, not general glob patterns.
			// Leave the picker unrestricted if any pattern cannot be represented.
			let extension = pattern.strip_prefix("*.")?;
			if extension.is_empty() || extension.contains(['*', '?', '[', ']', '{', '}', '/', '\\']) {
				return None;
			}
			if !result.iter().any(|item| item == extension) {
				result.push(extension.to_string());
			}
		}
	}
	if result.is_empty() { None } else { Some(result) }
}

fn initial_directory(initial_path: Option<&Path>) -> Option<PathBuf> {
	initial_directory_and_name(initial_path).0
}

fn initial_directory_and_name(initial_path: Option<&Path>) -> (Option<PathBuf>, Option<String>) {
	let Some(path) = utils::abspath(initial_path) else {
		return (None, None);
	};

	if path.is_dir() {
		return (Some(path.into_owned()), None);
	}

	let directory = path.parent().map(Path::to_path_buf);
	let file_name = path.file_name().and_then(|name| name.to_str()).map(str::to_string);
	(directory, file_name)
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

#[cfg(test)]
mod tests {
	use super::*;

	const ECHO: &str = "on run argv\nreturn item 1 of argv\nend run";

	#[test]
	fn script_arguments_are_literal() {
		for value in ["-literal-title", "--", "café Ω \"quoted\" \\"] {
			assert_eq!(invoke_output(ECHO, &[value]).as_deref(), Some(value));
		}
	}

	#[test]
	fn output_preserves_whitespace() {
		for value in ["", "  text\t  ", "text\n", "text\r", "text\r\n\n"] {
			assert_eq!(invoke_output(ECHO, &[value]).as_deref(), Some(value));
		}
	}

	#[test]
	fn path_results_preserve_line_breaks() {
		let paths = ["/tmp/a\nb.txt\n", "/tmp/café \"quoted\"\r"];
		let script = "on run argv\nset AppleScript's text item delimiters to character id 0\nreturn argv as text\nend run";
		let output = invoke_output(script, &paths).unwrap();
		assert_eq!(parse_paths(&output), Some(paths.map(PathBuf::from).to_vec()));
		assert_eq!(parse_paths(""), None);
	}

	#[test]
	fn extension_filters_are_combined() {
		let filters = [FileFilter { name: "Sources", patterns: &["*.toml", "*.md", "*.toml"] }];
		assert_eq!(filter_types(Some(&filters)), Some(vec!["toml".into(), "md".into()]));
		assert_eq!(filter_types(None), None);
		assert_eq!(filter_types(Some(&[])), None);
	}

	#[test]
	fn unsupported_filters_leave_the_picker_unrestricted() {
		for pattern in ["*", "*.*", "Cargo.toml", "data-*.txt", "*.t?xt", "*.[ch]"] {
			let filters = [FileFilter { name: "Files", patterns: &["*.md", pattern] }];
			assert_eq!(filter_types(Some(&filters)), None, "{pattern}");
		}
	}
}
