use super::*;

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	choose_folders_impl(p, false).and_then(|paths| paths.into_iter().next())
}

pub fn choose_folders(p: &FolderDialog<'_>) -> Option<Vec<PathBuf>> {
	choose_folders_impl(p, true)
}

fn choose_folders_impl(p: &FolderDialog<'_>, multiple: bool) -> Option<Vec<PathBuf>> {
	ensure_gtk_initialized();

	let title = cstring(p.title);
	let accept = c"Select";
	let cancel = c"Cancel";

	let native = unsafe {
		gtk_sys::gtk_file_chooser_native_new(
			title.as_ptr(),
			ptr::null_mut(),
			gtk_sys::GTK_FILE_CHOOSER_ACTION_SELECT_FOLDER,
			accept.as_ptr(),
			cancel.as_ptr(),
		)
	};
	let chooser = native as *mut gtk_sys::GtkFileChooser;

	unsafe {
		gtk_sys::gtk_file_chooser_set_select_multiple(chooser, multiple as i32);
		if let Some(directory) = p.directory {
			let c_path = cstring(directory.to_string_lossy().as_ref());
			gtk_sys::gtk_file_chooser_set_current_folder(chooser, c_path.as_ptr());
		}
	}

	let response = run_native_dialog(native as *mut gtk_sys::GtkNativeDialog);
	if response != gtk_sys::GTK_RESPONSE_ACCEPT {
		unsafe { g_object_unref(native as *mut _) };
		return None;
	}

	let result = if multiple {
		let list = unsafe { gtk_sys::gtk_file_chooser_get_filenames(chooser) };
		if list.is_null() {
			Vec::new()
		}
		else {
			collect_file_list(list)
		}
	}
	else {
		let filename = unsafe { gtk_sys::gtk_file_chooser_get_filename(chooser) };
		c_to_path_buf(filename).into_iter().collect()
	};

	unsafe { g_object_unref(native as *mut _) };
	Some(result)
}
