use std::fmt::Write;
use std::path::{Path, PathBuf};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;

use super::{
	MessageIcon, MessageButtons, MessageResult, MessageBox,
	FileFilter, FileDialog, FolderDialog,
	TextInputMode, TextInput,
	ColorValue, ColorPicker,
	NotifyPopup,
};
use super::utils;

mod file;
// mod file_com;
mod folder;
// mod folder_com;
mod input;
mod message;
mod color;
mod notify;

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

pub fn message_box(p: &MessageBox<'_>) -> Option<MessageResult> {
	message::show(p)
}

pub fn pick_file(p: &FileDialog<'_>) -> Option<std::path::PathBuf> {
	file::pick_file(p)
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<std::path::PathBuf>> {
	file::pick_files(p)
}

pub fn save_file(p: &FileDialog<'_>) -> Option<std::path::PathBuf> {
	file::save_file(p)
}

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<std::path::PathBuf> {
	folder::folder_dialog(p)
}

pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	input::text_input(p)
}

pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	color::color_picker(p)
}

pub fn notify_popup(p: &NotifyPopup<'_>) {
	notify::notify_popup(p)
}
