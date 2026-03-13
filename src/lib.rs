/*!
Rusty Dialogs is a cross-platform library for showing native dialog boxes and notifications.

Supported platforms and backends:

### Windows

Extensively tested on Windows 10 and 11.

- Win32-based legacy dialogs compatible with any COM apartment model.

- By default, notifications use a tray icon with balloon tips.

- Optional WinRT-Toast notifications are available on Windows 10 and later. (feature: `winrt-toast`)

### Linux & BSDs

Extensively tested on Linux Ubuntu 24 LTS.

- By default, executable-based backends (`kdialog` and `zenity`) are used.

- Optional GTK3 and GTK4 backends are available with libnotify-based notifications. (feature: `gtk3`, `gtk4`)

- XDG desktop portal support is also available, but limited to file and folder dialogs. (feature: `xdg-portal`)

### macOS

Untested on macOS. No test report yet.

- By default, AppleScript-based dialogs are used.

- Optional AppKit-based dialogs and notifications are also available. (feature: `appkit`)

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
	///
	/// Note: Some platforms/backends/dialogs may not have a distinct question icon and may use the information icon instead.
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
	/// Prefer to check if the dialog result matches `Some(Ok)` or `Some(Yes)` rather than checking for `Some(No)`, `Some(Cancel)` or `None`.
	///
	/// When the dialog is dismissed (press Cancel, close the dialog, or pressing ESC), the result maybe `None` or may be any of the message results even if that button is not present (e.g. `Some(Cancel)`).
	#[inline]
	pub fn show(&self) -> Option<MessageResult> {
		message_box(self)
	}
}


/// File filter for file dialogs.
#[derive(Copy, Clone, Debug)]
pub struct FileFilter<'a> {
	/// The description of the file filter, e.g. `"Text Files"`.
	pub name: &'a str,
	/// The file patterns of the file filter, e.g. `&["*.txt"]` or `&["*.jpg", "*.jpeg"]`.
	pub patterns: &'a [&'a str],
}

#[allow(dead_code)]
impl FileFilter<'_> {
	const ALL_FILES: FileFilter<'static> = FileFilter {
		name: "All Files",
		patterns: &["*"],
	};
}

/// File dialog.
///
/// The file dialog allows the user to select a file or multiple files, specify a file name for saving, or select folders.
///
/// ```no_run
/// use std::env;
///
/// let file = rustydialogs::FileDialog {
/// 	title: "Open File",
/// 	path: env::current_dir().ok().as_deref(),
/// 	filters: Some(&[
/// 		rustydialogs::FileFilter {
/// 			name: "Text Files",
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
	///
	/// An additional "All Files" filter is automatically added to the end of the list.
	pub filters: Option<&'a [FileFilter<'a>]>,
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

	/// Show folder picker dialog, allowing the user to select a single folder.
	///
	/// The `filters` field is ignored for folder selection.
	#[inline]
	pub fn choose_folder(&self) -> Option<PathBuf> {
		choose_folder(self)
	}

	/// Show folder picker dialog, allowing the user to select multiple folders.
	///
	/// The `filters` field is ignored for folder selection.
	///
	/// ### Platform-specific behavior
	///
	/// Backends that do not support selecting multiple folders fall back to a single-folder picker:
	///
	/// - Linux: `kdialog`.
	#[inline]
	pub fn choose_folders(&self) -> Option<Vec<PathBuf>> {
		choose_folders(self)
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

/// Notification duration for notifications.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NotifyDuration {
	/// Short duration for notifications, typically around 5 seconds.
	Short,
	/// Long duration for notifications, typically around 10 seconds.
	Long,
	/// Infinite duration for notifications, which do not automatically close.
	Infinite,
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
/// 	duration: rustydialogs::NotifyDuration::Short,
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
	/// The timeout duration for the notification popup.
	///
	/// This is a best-effort hint: some backends may ignore it and use their own default timeout, or may not support timeouts at all.
	pub duration: NotifyDuration,
}

impl<'a> Notification<'a> {
	/// Perform any necessary setup for notifications, such as registering the application with the notification system.
	///
	/// This step is optional, when skipped the library will attempt to perform any necessary setup automatically when showing the first notification,
	/// but this method can be used to ensure that the setup is done at a specific time in the application lifecycle.
	///
	/// Returns `true` if the setup was successful or is not needed, and `false` if the setup failed and notifications may not work properly.
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
	pub fn setup(app_id: &str) -> bool {
		notify_setup(app_id)
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
