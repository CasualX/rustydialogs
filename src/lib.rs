use std::path::Path;
use raw_window_handle::HasWindowHandle;

mod utils;

/// Icon types for message dialogs.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MessageIcon {
	/// Information icon.
	Info,
	/// Warning icon.
	Warning,
	/// Error icon.
	Error,
	/// Question icon.
	Question,
}

/// Button configurations for message dialogs.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MessageButtons {
	/// OK button only.
	Ok,
	/// OK and Cancel buttons.
	OkCancel,
	/// Yes and No buttons.
	YesNo,
	/// Yes, No, and Cancel buttons.
	YesNoCancel,
}

/// Result of a message dialog.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MessageResult {
	/// OK result.
	Ok,
	/// Cancel result.
	Cancel,
	/// Yes result.
	Yes,
	/// No result.
	No,
}

/// Message box dialog.
#[derive(Copy, Clone)]
pub struct MessageBox<'a> {
	/// The title of the dialog.
	pub title: &'a str,
	/// The message to display to the user.
	pub message: &'a str,
	/// The icon to show in the dialog.
	pub icon: MessageIcon,
	/// The buttons to show in the dialog.
	pub buttons: MessageButtons,
	/// The owner window of the dialog.
	pub owner: Option<&'a dyn HasWindowHandle>,
}

impl<'a> MessageBox<'a> {
	/// Show the dialog.
	///
	/// # Linux
	///
	/// When the window is dismissed by clicking the close button, the result is `None`.
	///
	/// # Windows
	///
	/// When the window is dismissed by clicking the close button, the result is Some with the rejective button, e.g. `Some(MessageResult::Cancel)` for [`MessageButtons::OkCancel`] or `Some(MessageResult::No)` for [`MessageButtons::YesNo`].
	pub fn show(&self) -> Option<MessageResult> {
		message_box(self)
	}
}


/// File filter for file dialogs.
#[derive(Copy, Clone, Debug)]
pub struct FileFilter<'a> {
	/// The description of the file filter, e.g. `"Text Files"`.
	pub desc: &'a str,
	/// The file patterns of the file filter, e.g. `&["*.txt"]` or `&["*.jpg", "*.jpeg"]`.
	pub patterns: &'a [&'a str],
}

/// File dialog.
///
/// The file dialog allows the user to select a file or multiple files, or to specify a file name for saving.
#[derive(Copy, Clone)]
pub struct FileDialog<'a> {
	/// The title of the dialog.
	pub title: &'a str,
	/// The initial directory to show in the file dialog.
	///
	/// If `file_name` is also provided, this is the parent directory of `file_name`.
	pub directory: Option<&'a Path>,
	/// The initial file name to show in the file dialog.
	///
	/// If `directory` is also provided, this is joined with `directory` to form the full initial path and file name.
	pub file_name: Option<&'a Path>,
	/// An optional list of file filters to show in the file dialog.
	pub filter: Option<&'a [FileFilter<'a>]>,
	/// The owner window of the dialog.
	pub owner: Option<&'a dyn HasWindowHandle>,
}

impl<'a> FileDialog<'a> {
	/// Show open file dialog, allowing the user to select a single file.
	pub fn pick_file(&self) -> Option<std::path::PathBuf> {
		pick_file(self)
	}

	/// Show open file dialog, allowing the user to select multiple files.
	pub fn pick_files(&self) -> Option<Vec<std::path::PathBuf>> {
		pick_files(self)
	}

	/// Show save file dialog.
	pub fn save_file(&self) -> Option<std::path::PathBuf> {
		save_file(self)
	}
}

/// Folder dialog.
///
/// The folder dialog allows the user to select a folder or directory.
#[derive(Copy, Clone)]
pub struct FolderDialog<'a> {
	/// The title of the dialog.
	pub title: &'a str,
	/// The initial directory to show in the folder dialog.
	pub directory: Option<&'a Path>,
	/// The owner window of the dialog.
	pub owner: Option<&'a dyn HasWindowHandle>,
}

impl<'a> FolderDialog<'a> {
	/// Show the dialog.
	pub fn show(&self) -> Option<std::path::PathBuf> {
		folder_dialog(self)
	}
}

/// Modes for text input dialogs.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TextInputMode {
	/// Single line text input dialog.
	SingleLine,
	/// Multi-line text input dialog.
	MultiLine,
	/// Password input dialog, which hides the input text.
	Password,
}

/// Text input dialog.
///
/// The text input dialog allows the user to enter text, which is returned as a string.
#[derive(Copy, Clone)]
pub struct TextInput<'a> {
	/// The title of the dialog.
	pub title: &'a str,
	/// The message to display to the user.
	pub message: &'a str,
	/// The initial value to display in the text input.
	pub value: &'a str,
	/// The mode of the text input, which determines the type of dialog shown and how the input is handled.
	pub mode: TextInputMode,
	/// The owner window of the dialog.
	pub owner: Option<&'a dyn HasWindowHandle>,
}

impl<'a> TextInput<'a> {
	/// Show the dialog.
	///
	/// Returns `Some(String)` if the user provided input and confirmed the dialog, or `None` if the user cancelled the dialog.
	pub fn show(&self) -> Option<String> {
		text_input(self)
	}
}

/// Color value.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ColorValue {
	/// The red component of the color, in the range [0, 255].
	pub red: u8,
	/// The green component of the color, in the range [0, 255].
	pub green: u8,
	/// The blue component of the color, in the range [0, 255].
	pub blue: u8,
}

/// Color picker dialog.
///
/// The color picker dialog allows the user to select a color, which is returned as an RGB value.
/// The dialog may also show a palette of predefined colors for the user to choose from.
#[derive(Copy, Clone)]
pub struct ColorPicker<'a> {
	/// The title of the dialog.
	pub title: &'a str,
	/// The initial color value to show in the color picker dialog.
	pub value: ColorValue,
	/// The owner window of the dialog.
	pub owner: Option<&'a dyn HasWindowHandle>,
}

impl<'a> ColorPicker<'a> {
	/// Show the dialog.
	///
	/// Returns `Some(ColorValue)` if the user selected a color and confirmed the dialog, or `None` if the user cancelled the dialog.
	pub fn show(&self) -> Option<ColorValue> {
		color_picker(self)
	}
}

/// Notification popup.
///
/// Shows a brief message to the user without blocking their interaction with the application.
#[derive(Copy, Clone, Debug)]
pub struct NotifyPopup<'a> {
	/// The title of the notification popup.
	pub title: &'a str,
	/// The message to display in the notification popup.
	pub message: &'a str,
	/// The icon to show in the notification popup.
	pub icon: MessageIcon,
	/// Timeout in milliseconds after which the notification should automatically close.
	///
	/// A value less than or equal to `0` means that the notification will not automatically close.
	pub timeout: i32,
}

impl<'a> NotifyPopup<'a> {
	/// Show the notification popup.
	pub fn show(&self) {
		notify_popup(self)
	}
}

#[cfg(any(
	target_os = "linux",
	target_os = "freebsd",
	target_os = "dragonfly",
	target_os = "netbsd",
	target_os = "openbsd",
))]
mod linux;
#[cfg(any(
	target_os = "linux",
	target_os = "freebsd",
	target_os = "dragonfly",
	target_os = "netbsd",
	target_os = "openbsd",
))]
use linux::*;

#[cfg(windows)]
mod win32;
#[cfg(windows)]
use win32::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::*;
