use std::ffi::{CStr, CString, OsStr};
use std::os::raw::{c_char, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::{ptr, sync};

use gtk4_gio_sys::{g_file_get_path, g_list_model_get_item, g_list_model_get_n_items, GFile, GListModel};
use gtk4_glib_sys::{g_free, g_main_loop_new, g_main_loop_quit, g_main_loop_run, g_main_loop_unref, GMainLoop};
use gtk4_gobject_sys::{g_object_unref, g_signal_connect_data};

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
	let ok = *GTK_INITIALIZED.get_or_init(|| unsafe { gtk4_sys::gtk_init_check() != 0 });
	if !ok {
		panic!("Failed to initialize GTK4 backend. Ensure a graphical session is available.");
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

fn gfile_to_path_buf(file: *mut GFile) -> Option<PathBuf> {
	if file.is_null() {
		return None;
	}
	let path_ptr = unsafe { g_file_get_path(file) };
	unsafe { g_object_unref(file as *mut _) };
	c_to_path_buf(path_ptr)
}

fn collect_file_model(model: *mut GListModel) -> Vec<PathBuf> {
	if model.is_null() {
		return Vec::new();
	}

	let mut result = Vec::new();
	let count = unsafe { g_list_model_get_n_items(model) };
	for index in 0..count {
		let item = unsafe { g_list_model_get_item(model, index) };
		let file = item as *mut GFile;
		if let Some(path) = gfile_to_path_buf(file) {
			result.push(path);
		}
	}

	unsafe { g_object_unref(model as *mut _) };
	result
}

#[repr(C)]
struct ResponseState {
	loop_: *mut GMainLoop,
	response: i32,
}

unsafe extern "C" fn on_dialog_response(
	_dialog: *mut gtk4_sys::GtkDialog,
	response: i32,
	user_data: *mut c_void,
) {
	let state = &mut *(user_data as *mut ResponseState);
	state.response = response;
	g_main_loop_quit(state.loop_);
}

unsafe extern "C" fn on_native_dialog_response(
	_dialog: *mut gtk4_sys::GtkNativeDialog,
	response: i32,
	user_data: *mut c_void,
) {
	let state = &mut *(user_data as *mut ResponseState);
	state.response = response;
	g_main_loop_quit(state.loop_);
}

fn run_dialog(dialog: *mut gtk4_sys::GtkDialog) -> i32 {
	let loop_ = unsafe { g_main_loop_new(ptr::null_mut(), 0) };
	let mut state = ResponseState {
		loop_,
		response: gtk4_sys::GTK_RESPONSE_NONE,
	};

	unsafe {
		let callback: gtk4_gobject_sys::GCallback = std::mem::transmute(Some(
			on_dialog_response
				as unsafe extern "C" fn(*mut gtk4_sys::GtkDialog, i32, *mut c_void),
		));
		g_signal_connect_data(
			dialog as *mut _,
			c"response".as_ptr(),
			callback,
			&mut state as *mut _ as *mut _,
			None,
			0,
		);

		gtk4_sys::gtk_window_present(dialog as *mut gtk4_sys::GtkWindow);
		g_main_loop_run(loop_);
		g_main_loop_unref(loop_);
	}

	state.response
}

fn run_native_dialog(dialog: *mut gtk4_sys::GtkNativeDialog) -> i32 {
	let loop_ = unsafe { g_main_loop_new(ptr::null_mut(), 0) };
	let mut state = ResponseState {
		loop_,
		response: gtk4_sys::GTK_RESPONSE_NONE,
	};

	unsafe {
		let callback: gtk4_gobject_sys::GCallback = std::mem::transmute(Some(
			on_native_dialog_response
				as unsafe extern "C" fn(*mut gtk4_sys::GtkNativeDialog, i32, *mut c_void),
		));
		g_signal_connect_data(
			dialog as *mut _,
			c"response".as_ptr(),
			callback,
			&mut state as *mut _ as *mut _,
			None,
			0,
		);

		gtk4_sys::gtk_native_dialog_show(dialog);
		g_main_loop_run(loop_);
		gtk4_sys::gtk_native_dialog_hide(dialog);
		g_main_loop_unref(loop_);
	}

	state.response
}

fn file_path(directory: Option<&Path>, file_name: Option<&Path>) -> Option<PathBuf> {
	match (directory, file_name) {
		(Some(dir), Some(file)) => Some(dir.join(file)),
		(Some(dir), None) => Some(dir.to_path_buf()),
		(None, Some(file)) => Some(file.to_path_buf()),
		(None, None) => None,
	}
}
