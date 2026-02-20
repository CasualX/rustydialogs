use std::{path, process, str, sync};
use std::ffi::{OsStr, OsString};
use std::fmt::Write;
use std::os::unix::ffi::OsStrExt;

use super::*;

mod kdialog;
mod zenity;

#[cfg(feature = "gtk3")]
mod gtk3;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Backend {
	KDialog,
	Zenity,
	#[cfg(feature = "gtk3")]
	Gtk3,
}

static BACKEND: sync::LazyLock<Backend> = sync::LazyLock::new(|| {
	fn getenv(key: &std::ffi::CStr) -> Option<&[u8]> {
		unsafe {
			let ptr = libc::getenv(key.as_ptr());
			if ptr.is_null() {
				None
			}
			else {
				Some(std::ffi::CStr::from_ptr(ptr).to_bytes())
			}
		}
	}

	// Check RUSTY_DIALOGS_BACKEND env var first, then check for kdialog and zenity executables.
	if let Some(backend) = getenv(c"RUSTY_DIALOGS_BACKEND") {
		match backend {
			b"kdialog" => return Backend::KDialog,
			b"zenity" => return Backend::Zenity,
			#[cfg(feature = "gtk3")]
			b"gtk3" => return Backend::Gtk3,
			_ => panic!("Invalid RUSTY_DIALOGS_BACKEND value: {backend:?}", backend = str::from_utf8(backend).unwrap_or("<invalid utf-8>")),
		}
	}

	#[cfg(feature = "gtk3")] {
		return Backend::Gtk3;
	}

	#[cfg(not(feature = "gtk3"))] {
		fn isenv(key: &std::ffi::CStr) -> bool {
			unsafe { !libc::getenv(key.as_ptr()).is_null() }
		}

		let desktop = getenv(c"XDG_CURRENT_DESKTOP").or_else(|| getenv(c"DESKTOP_SESSION")).and_then(|s| str::from_utf8(s).ok());
		let preferred_programs = if let Some(desktop) = desktop {
			if desktop.contains("gnome") {
				[Backend::Zenity, Backend::KDialog]
			}
			else if desktop.contains("kde") || desktop.contains("plasma") {
				[Backend::KDialog, Backend::Zenity]
			}
			else {
				[Backend::Zenity, Backend::KDialog]
			}
		}
		else if isenv(c"GNOME_DESKTOP_SESSION_ID") {
			[Backend::Zenity, Backend::KDialog]
		}
		else {
			[Backend::KDialog, Backend::Zenity]
		};

		// Run 'which program' for each program and return the first one that exists.
		for &backend in &preferred_programs {
			let program = match backend {
				Backend::KDialog => "kdialog",
				Backend::Zenity => "zenity",
			};
			if process::Command::new("which").arg(program).output().map(|output| output.status.success()).unwrap_or(false) {
				return backend;
			}
		}
		panic!("No supported dialog backend found. Please install kdialog or zenity, or set RUSTY_DIALOGS_BACKEND to a supported backend.");
	}
});


pub fn message_box(p: &MessageBox<'_>) -> Option<MessageResult> {
	match *BACKEND {
		Backend::KDialog => kdialog::message_box(p),
		Backend::Zenity => zenity::message_box(p),
		#[cfg(feature = "gtk3")]
		Backend::Gtk3 => gtk3::message_box(p),
	}
}

pub fn pick_file(p: &FileDialog<'_>) -> Option<path::PathBuf> {
	match *BACKEND {
		Backend::KDialog => kdialog::pick_file(p),
		Backend::Zenity => zenity::pick_file(p),
		#[cfg(feature = "gtk3")]
		Backend::Gtk3 => gtk3::pick_file(p),
	}
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<path::PathBuf>> {
	match *BACKEND {
		Backend::KDialog => kdialog::pick_files(p),
		Backend::Zenity => zenity::pick_files(p),
		#[cfg(feature = "gtk3")]
		Backend::Gtk3 => gtk3::pick_files(p),
	}
}

pub fn save_file(p: &FileDialog<'_>) -> Option<path::PathBuf> {
	match *BACKEND {
		Backend::KDialog => kdialog::save_file(p),
		Backend::Zenity => zenity::save_file(p),
		#[cfg(feature = "gtk3")]
		Backend::Gtk3 => gtk3::save_file(p),
	}
}

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<path::PathBuf> {
	match *BACKEND {
		Backend::KDialog => kdialog::folder_dialog(p),
		Backend::Zenity => zenity::folder_dialog(p),
		#[cfg(feature = "gtk3")]
		Backend::Gtk3 => gtk3::folder_dialog(p),
	}
}

pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	match *BACKEND {
		Backend::KDialog => kdialog::text_input(p),
		Backend::Zenity => zenity::text_input(p),
		#[cfg(feature = "gtk3")]
		Backend::Gtk3 => gtk3::text_input(p),
	}
}

pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	match *BACKEND {
		Backend::KDialog => kdialog::color_picker(p),
		Backend::Zenity => zenity::color_picker(p),
		#[cfg(feature = "gtk3")]
		Backend::Gtk3 => gtk3::color_picker(p),
	}
}

pub fn notify_popup(p: &NotifyPopup<'_>) {
	match *BACKEND {
		Backend::KDialog => kdialog::notify_popup(p),
		Backend::Zenity => zenity::notify_popup(p),
		#[cfg(feature = "gtk3")]
		Backend::Gtk3 => gtk3::notify_popup(p),
	}
}

#[inline]
fn os(s: &str) -> &OsStr {
	OsStr::new(s)
}

#[track_caller]
fn invoke(program: &str, args: &[&OsStr]) -> Option<i32> {
	let mut child = process::Command::new(program).args(args).spawn().unwrap();
	child.wait().unwrap().code()
}

#[track_caller]
fn invoke_async(program: &str, args: &[&OsStr]) {
	let _ = process::Command::new(program).args(args).spawn().unwrap();
}

#[track_caller]
fn invoke_output(program: &str, args: &[&OsStr]) -> (Option<i32>, String) {
	let output = process::Command::new(program).args(args).output().unwrap();
	let stdout = String::from_utf8(output.stdout)
		.unwrap_or_else(|err| String::from_utf8_lossy(err.as_bytes()).to_string());
	let code = output.status.code();
	(code, stdout)
}

#[track_caller]
fn invoke_output_bytes(program: &str, args: &[&OsStr]) -> (Option<i32>, Vec<u8>) {
	let output = process::Command::new(program).args(args).output().unwrap();
	(output.status.code(), output.stdout)
}

#[track_caller]
fn exit_status_error(status: Option<i32>) -> ! {
	if let Some(code) = status {
		panic!("terminated with exit code: {code}")
	}
	else {
		panic!("terminated without an exit code")
	}
}
