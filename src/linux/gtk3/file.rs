use super::*;

fn apply_file_chooser_defaults(dialog: *mut gtk_sys::GtkFileChooser, p: &FileDialog<'_>, save: bool, multiple: bool) {
	unsafe {
		gtk_sys::gtk_file_chooser_set_select_multiple(dialog, multiple as i32);
		if save {
			gtk_sys::gtk_file_chooser_set_do_overwrite_confirmation(dialog, 1);
		}

		if let Some(path) = file_path(p.directory, p.file_name) {
			let c_path = cstring(path.to_string_lossy().as_ref());
			gtk_sys::gtk_file_chooser_set_filename(dialog, c_path.as_ptr());
		}

		if let Some(filters) = p.filter {
			for entry in filters {
				let filter = gtk_sys::gtk_file_filter_new();
				let desc = cstring(entry.desc);
				gtk_sys::gtk_file_filter_set_name(filter, desc.as_ptr());
				for pattern in entry.patterns {
					let pattern = cstring(pattern);
					gtk_sys::gtk_file_filter_add_pattern(filter, pattern.as_ptr());
				}
				gtk_sys::gtk_file_chooser_add_filter(dialog, filter);
			}
		}

		let all_files_filter = gtk_sys::gtk_file_filter_new();
		let all_files_name = c"All Files";
		let all_files_pattern = c"*";
		gtk_sys::gtk_file_filter_set_name(all_files_filter, all_files_name.as_ptr());
		gtk_sys::gtk_file_filter_add_pattern(all_files_filter, all_files_pattern.as_ptr());
		gtk_sys::gtk_file_chooser_add_filter(dialog, all_files_filter);
	}
}

pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	pick_files_impl(p, false).and_then(|files| files.into_iter().next())
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	pick_files_impl(p, true)
}

fn pick_files_impl(p: &FileDialog<'_>, multiple: bool) -> Option<Vec<PathBuf>> {
	ensure_gtk_initialized();

	let title = cstring(p.title);
	let accept = c"Open";
	let cancel = c"Cancel";

	let native = unsafe {
		gtk_sys::gtk_file_chooser_native_new(
			title.as_ptr(),
			ptr::null_mut(),
			gtk_sys::GTK_FILE_CHOOSER_ACTION_OPEN,
			accept.as_ptr(),
			cancel.as_ptr(),
		)
	};
	let chooser = native as *mut gtk_sys::GtkFileChooser;

	apply_file_chooser_defaults(chooser, p, false, multiple);

	let response = unsafe { gtk_sys::gtk_native_dialog_run(native as *mut gtk_sys::GtkNativeDialog) };
	if response != gtk_sys::GTK_RESPONSE_ACCEPT {
		unsafe { g_object_unref(native as *mut _) };
		return None;
	}

	let result = if multiple {
		let list = unsafe { gtk_sys::gtk_file_chooser_get_filenames(chooser) };
		if list.is_null() {
			Vec::new()
		} else {
			collect_file_list(list)
		}
	} else {
		let filename = unsafe { gtk_sys::gtk_file_chooser_get_filename(chooser) };
		c_to_path_buf(filename).into_iter().collect()
	};

	unsafe { g_object_unref(native as *mut _) };
	Some(result)
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	ensure_gtk_initialized();

	let title = cstring(p.title);
	let accept = c"Save";
	let cancel = c"Cancel";

	let native = unsafe {
		gtk_sys::gtk_file_chooser_native_new(
			title.as_ptr(),
			ptr::null_mut(),
			gtk_sys::GTK_FILE_CHOOSER_ACTION_SAVE,
			accept.as_ptr(),
			cancel.as_ptr(),
		)
	};
	let chooser = native as *mut gtk_sys::GtkFileChooser;

	apply_file_chooser_defaults(chooser, p, true, false);

	let response = unsafe { gtk_sys::gtk_native_dialog_run(native as *mut gtk_sys::GtkNativeDialog) };
	if response != gtk_sys::GTK_RESPONSE_ACCEPT {
		unsafe { g_object_unref(native as *mut _) };
		return None;
	}

	let filename = unsafe { gtk_sys::gtk_file_chooser_get_filename(chooser) };
	let result = c_to_path_buf(filename);

	unsafe { g_object_unref(native as *mut _) };
	result
}
