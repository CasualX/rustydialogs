use std::fmt::Write;
use super::*;

mod file;
mod folder;
mod input;
mod message;
mod color;
mod notify;

fn utf16_null_terminated(value: &str) -> Vec<u16> {
	value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn show(p: &MessageBox<'_>) -> Option<MessageResult> {
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
