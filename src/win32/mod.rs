use std::fmt::Write;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;

use super::*;

mod file;
// mod file_com;
mod folder;
// mod folder_com;
mod input;
mod message;
mod color;
#[cfg(feature = "winrt-toast")]
mod toast;
#[cfg(not(feature = "winrt-toast"))]
mod tray;

fn utf16cs(value: &str) -> Vec<u16> {
	let mut encoded = Vec::with_capacity(value.len() + 1);
	encoded.extend(value.encode_utf16());
	encoded.push(0);
	encoded
}

fn hwnd(owner: Option<&dyn HasWindowHandle>) -> Option<HWND> {
	let raw = owner.and_then(|w| w.window_handle().ok()).map(|h| h.as_raw());
	match raw {
		Some(RawWindowHandle::Win32(handle)) => Some(HWND(handle.hwnd.get() as *mut core::ffi::c_void)),
		_ => None,
	}
}

#[inline]
pub fn message_box(p: &MessageBox<'_>) -> Option<MessageResult> {
	message::show(p)
}

#[inline]
pub fn pick_file(p: &FileDialog<'_>) -> Option<std::path::PathBuf> {
	file::pick_file(p)
}

#[inline]
pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<std::path::PathBuf>> {
	file::pick_files(p)
}

#[inline]
pub fn save_file(p: &FileDialog<'_>) -> Option<std::path::PathBuf> {
	file::save_file(p)
}

#[inline]
pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<std::path::PathBuf> {
	folder::folder_dialog(p)
}

#[inline]
pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	input::text_input(p)
}

#[inline]
pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	color::color_picker(p)
}

#[inline]
pub fn notify_setup(app_id: &str) -> bool {
	#[cfg(feature = "winrt-toast")] {
		toast::setup(app_id)
	}
	#[cfg(not(feature = "winrt-toast"))] {
		tray::setup(app_id)
	}
}

#[inline]
pub fn notify(p: &Notification<'_>) {
	#[cfg(feature = "winrt-toast")] {
		toast::notify(p)
	}
	#[cfg(not(feature = "winrt-toast"))] {
		tray::notify(p)
	}
}
