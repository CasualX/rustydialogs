use super::*;

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	ensure_gtk_initialized();

	let title = cstring(p.title);
	let accept = c"Select";
	let cancel = c"Cancel";

	let native = unsafe {
		gtk4_sys::gtk_file_chooser_native_new(
			title.as_ptr(),
			ptr::null_mut(),
			gtk4_sys::GTK_FILE_CHOOSER_ACTION_SELECT_FOLDER,
			accept.as_ptr(),
			cancel.as_ptr(),
		)
	};
	let chooser = native as *mut gtk4_sys::GtkFileChooser;

	unsafe {
		if let Some(directory) = p.directory {
			let c_path = cstring(directory.to_string_lossy().as_ref());
			let file = gtk4_gio_sys::g_file_new_for_path(c_path.as_ptr());
			gtk4_sys::gtk_file_chooser_set_current_folder(chooser, file, ptr::null_mut());
			g_object_unref(file as *mut _);
		}
	}

	let response = run_native_dialog(native as *mut gtk4_sys::GtkNativeDialog);
	if response != gtk4_sys::GTK_RESPONSE_ACCEPT {
		unsafe { g_object_unref(native as *mut _) };
		return None;
	}

	let file = unsafe { gtk4_sys::gtk_file_chooser_get_file(chooser) };
	let result = gfile_to_path_buf(file as *mut GFile);

	unsafe { g_object_unref(native as *mut _) };
	result
}
