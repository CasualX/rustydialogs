/*!
Rusty Dialogs is a cross-platform library for showing native dialog boxes and notifications.

Supported platforms and backends:

### Windows

Win32-based legacy dialogs compatible with any COM apartment model.

By default, notifications use a tray icon with balloon tips.

Optional WinRT-Toast notifications are available on Windows 10 and later. (feature: `winrt-toast`)

### Linux & BSDs

By default, executable-based backends (`kdialog` and `zenity`) are used.

Optional GTK3 and GTK4 backends are available with libnotify-based notifications. (feature: `gtk3`, `gtk4`)

XDG desktop portal support is also available, but limited to file and folder dialogs. (feature: `xdg-portal`)

### macOS

By default, AppleScript-based dialogs are used.

Optional AppKit-based dialogs and notifications are also available. (feature: `appkit`)
*/

use std::path::{Path, PathBuf};
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
///
/// ```no_run
/// let result = rustydialogs::MessageBox {
/// 	title: "Confirm Action",
/// 	message: "Are you sure you want to proceed?",
/// 	icon: rustydialogs::MessageIcon::Question,
/// 	buttons: rustydialogs::MessageButtons::YesNo,
/// 	owner: None,
/// }.show();
/// if result == Some(rustydialogs::MessageResult::Yes) {
/// 	println!("User chose Yes");
/// }
/// ```
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
	/// Prefer to check if the dialog result matches `Some(Ok)` or `Some(Yes)` rather than checking for `Some(No)` or `Some(Cancel)`.
	///
	/// When the dialog is dismissed (closing the dialog or pressing ESC), the result maybe `None` or may be any of the message results even if that button is not present (e.g. `Some(Cancel)`).
	#[inline]
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
///
/// ```no_run
/// use std::env;
///
/// let file = rustydialogs::FileDialog {
/// 	title: "Open File",
/// 	path: env::current_dir().ok().as_deref(),
/// 	filter: Some(&[
/// 		rustydialogs::FileFilter {
/// 			desc: "Text Files",
/// 			patterns: &["*.txt", "*.md"],
/// 		},
/// 	]),
/// 	owner: None,
/// }.pick_file();
///
/// if let Some(path) = file {
/// 	println!("Picked file: {}", path.display());
/// }
/// ```
#[derive(Copy, Clone)]
pub struct FileDialog<'a> {
	/// The title of the dialog.
	pub title: &'a str,
	/// The initial path to show in the file dialog.
	///
	/// If the path is relative, it is joined with the current working directory.
	///
	/// If the resulting path exists and is a directory, no default file name is provided.
	///
	/// Otherwise, [`Path::file_name`] is used as the default file name.
	pub path: Option<&'a Path>,
	/// An optional list of file filters to show in the file dialog.
	pub filter: Option<&'a [FileFilter<'a>]>,
	/// The owner window of the dialog.
	pub owner: Option<&'a dyn HasWindowHandle>,
}

impl<'a> FileDialog<'a> {
	/// Show open file dialog, allowing the user to select a single file.
	#[inline]
	pub fn pick_file(&self) -> Option<PathBuf> {
		pick_file(self)
	}

	/// Show open file dialog, allowing the user to select multiple files.
	#[inline]
	pub fn pick_files(&self) -> Option<Vec<PathBuf>> {
		pick_files(self)
	}

	/// Show save file dialog.
	#[inline]
	pub fn save_file(&self) -> Option<PathBuf> {
		save_file(self)
	}
}

/// Folder dialog.
///
/// The folder dialog allows the user to select a folder or directory.
///
/// ```no_run
/// use std::env;
///
/// let folder = rustydialogs::FolderDialog {
/// 	title: "Select Folder",
/// 	directory: env::current_dir().ok().as_deref(),
/// 	owner: None,
/// }.show();
///
/// if let Some(path) = folder {
/// 	println!("Picked folder: {}", path.display());
/// }
/// ```
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
	#[inline]
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
///
/// ```no_run
/// let name = rustydialogs::TextInput {
/// 	title: "User Name",
/// 	message: "Enter your name:",
/// 	value: "",
/// 	mode: rustydialogs::TextInputMode::SingleLine,
/// 	owner: None,
/// }.show();
///
/// if let Some(name) = name {
/// 	println!("Hello, {name}!");
/// }
/// ```
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
	#[inline]
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
///
/// ```no_run
/// let color = rustydialogs::ColorPicker {
/// 	title: "Pick a Color",
/// 	value: rustydialogs::ColorValue {
/// 		red: 64,
/// 		green: 128,
/// 		blue: 255,
/// 	},
/// 	owner: None,
/// }.show();
///
/// if let Some(color) = color {
/// 	println!("RGB({}, {}, {})", color.red, color.green, color.blue);
/// }
/// ```
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
	#[inline]
	pub fn show(&self) -> Option<ColorValue> {
		color_picker(self)
	}
}

/// Notification.
///
/// Shows a brief message to the user without blocking their interaction with the application.
///
/// ```no_run
/// // Define a unique application identifier for the notification system.
/// const APP_ID: &str = "com.example.myapp";
///
/// // Invoke setup at application initialization to ensure the application
/// // is registered and ready to show notifications later.
/// rustydialogs::Notification::setup(APP_ID);
///
/// rustydialogs::Notification {
/// 	app_id: APP_ID,
/// 	title: "Task Complete",
/// 	message: "All files were processed successfully.",
/// 	icon: rustydialogs::MessageIcon::Info,
/// 	timeout: rustydialogs::Notification::SHORT_TIMEOUT,
/// }.show();
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Notification<'a> {
	/// Application identifier used by notification backends.
	///
	/// This is a best-effort hint: some backends may ignore it, and some only honor the first value seen by the process/session.
	pub app_id: &'a str,
	/// The title of the notification.
	pub title: &'a str,
	/// The message to display in the notification.
	pub message: &'a str,
	/// The icon to show in the notification.
	// Future: Change to optional Option<MessageIcon>
	pub icon: MessageIcon,
	/// Timeout in milliseconds after which the notification should automatically close.
	///
	/// A value less than or equal to `0` means that the notification will not automatically close.
	///
	/// This is a best-effort hint: some backends may ignore it and use their own default timeout, or may not support timeouts at all.
	// Future: Change to dedicated duration type
	pub timeout: i32,
}

impl<'a> Notification<'a> {
	/// Short timeout duration in milliseconds for notification popups.
	pub const SHORT_TIMEOUT: i32 = 5000;
	/// Long timeout duration in milliseconds for notification popups.
	pub const LONG_TIMEOUT: i32 = 10000;

	/// Perform any necessary setup for notifications, such as registering the application with the notification system.
	///
	/// This step is optional, when skipped the library will attempt to perform any necessary setup automatically when showing the first notification,
	/// but this method can be used to ensure that the setup is done at a specific time in the application lifecycle.
	///
	/// ### Windows
	///
	/// By default, this initializes a process-wide tray icon used for balloon notifications.
	///
	/// When using the `winrt-toast` backend, this creates a Start Menu shortcut for the application with the provided application identifier, which is required for showing toast notifications on Windows.
	/// It is recommended to call this method during application initialization before showing any notifications or the first notification may be skipped due to delays in the shortcut creation process.
	///
	/// ### Linux
	///
	/// When using the `libnotify` backend, this registers the application with the notification system using the provided application identifier.
	#[inline]
	pub fn setup(app_id: &str) {
		// Future: Return whether setup was successful (API breaking change)
		notify_setup(app_id);
	}

	/// Show the notification.
	#[inline]
	pub fn show(&self) {
		notify(self)
	}
}

#[cfg(windows)]
mod win32;
#[cfg(windows)]
use win32::*;

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

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::*;

#[cfg(not(any(
	windows,
	target_os = "linux",
	target_os = "freebsd",
	target_os = "dragonfly",
	target_os = "netbsd",
	target_os = "openbsd",
	target_os = "macos",
)))]
mod unsupported;
#[cfg(not(any(
	windows,
	target_os = "linux",
	target_os = "freebsd",
	target_os = "dragonfly",
	target_os = "netbsd",
	target_os = "openbsd",
	target_os = "macos",
)))]
use unsupported::*;
