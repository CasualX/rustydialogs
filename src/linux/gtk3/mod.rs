use std::ffi::{CStr, CString, OsStr};
use std::os::raw::c_char;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::{ptr, sync};

use glib_sys::{g_free, g_slist_free, GSList};
use gobject_sys::g_object_unref;

use super::*;

mod color;
mod file;
mod folder;
mod input;
mod message;
mod notify;

pub fn message_box(p: &MessageBox<'_>) -> Option<MessageResult> {
	message::show(p)
}

pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	file::pick_file(p)
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	file::pick_files(p)
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	file::save_file(p)
}

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
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

static GTK_INITIALIZED: sync::OnceLock<bool> = sync::OnceLock::new();

fn ensure_gtk_initialized() {
	let ok = *GTK_INITIALIZED.get_or_init(|| unsafe { gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) != 0 });
	if !ok {
		panic!("Failed to initialize GTK3 backend. Ensure a graphical session is available.");
	}
}

fn cstring(value: &str) -> CString {
	CString::new(value).unwrap_or_else(|_| CString::new(value.replace('\0', " ")).unwrap())
}

fn c_to_path_buf(ptr: *mut c_char) -> Option<PathBuf> {
	if ptr.is_null() {
		return None;
	}
	let bytes = unsafe { CStr::from_ptr(ptr).to_bytes().to_vec() };
	unsafe { g_free(ptr as *mut _) };
	Some(PathBuf::from(OsStr::from_bytes(&bytes)))
}

fn run_dialog(dialog: *mut gtk_sys::GtkWidget) -> i32 {
	let response = unsafe { gtk_sys::gtk_dialog_run(dialog as *mut gtk_sys::GtkDialog) };
	unsafe {
		gtk_sys::gtk_widget_destroy(dialog);
		while gtk_sys::gtk_events_pending() != 0 {
			gtk_sys::gtk_main_iteration();
		}
	}
	response
}

fn collect_file_list(list: *mut GSList) -> Vec<PathBuf> {
	let mut result = Vec::new();
	let mut node = list;
	while !node.is_null() {
		let filename_ptr = unsafe { (*node).data as *mut c_char };
		if !filename_ptr.is_null() {
			let bytes = unsafe { CStr::from_ptr(filename_ptr).to_bytes() };
			result.push(PathBuf::from(OsStr::from_bytes(bytes)));
			unsafe { g_free(filename_ptr as *mut _) };
		}
		node = unsafe { (*node).next };
	}
	unsafe { g_slist_free(list) };
	result
}

fn file_path(directory: Option<&Path>, file_name: Option<&Path>) -> Option<PathBuf> {
	match (directory, file_name) {
		(Some(dir), Some(file)) => Some(dir.join(file)),
		(Some(dir), None) => Some(dir.to_path_buf()),
		(None, Some(file)) => Some(file.to_path_buf()),
		(None, None) => None,
	}
}
