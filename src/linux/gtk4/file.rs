use super::*;

fn apply_file_chooser_defaults(dialog: *mut gtk4_sys::GtkFileChooser, p: &FileDialog<'_>, save: bool, multiple: bool) {
	unsafe {
		gtk4_sys::gtk_file_chooser_set_select_multiple(dialog, multiple as i32);
		let _ = save;

		if let Some(path) = file_path(p.directory, p.file_name) {
			let c_path = cstring(path.to_string_lossy().as_ref());
			let file = gtk4_gio_sys::g_file_new_for_path(c_path.as_ptr());
			gtk4_sys::gtk_file_chooser_set_file(dialog, file, ptr::null_mut());
			g_object_unref(file as *mut _);
		}

		if let Some(filters) = p.filter {
			for entry in filters {
				let filter = gtk4_sys::gtk_file_filter_new();
				let desc = cstring(entry.desc);
				gtk4_sys::gtk_file_filter_set_name(filter, desc.as_ptr());
				for pattern in entry.patterns {
					let pattern = cstring(pattern);
					gtk4_sys::gtk_file_filter_add_pattern(filter, pattern.as_ptr());
				}
				gtk4_sys::gtk_file_chooser_add_filter(dialog, filter);
			}
		}

		let all_files_filter = gtk4_sys::gtk_file_filter_new();
		let all_files_name = c"All Files";
		let all_files_pattern = c"*";
		gtk4_sys::gtk_file_filter_set_name(all_files_filter, all_files_name.as_ptr());
		gtk4_sys::gtk_file_filter_add_pattern(all_files_filter, all_files_pattern.as_ptr());
		gtk4_sys::gtk_file_chooser_add_filter(dialog, all_files_filter);
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
		gtk4_sys::gtk_file_chooser_native_new(
			title.as_ptr(),
			ptr::null_mut(),
			gtk4_sys::GTK_FILE_CHOOSER_ACTION_OPEN,
			accept.as_ptr(),
			cancel.as_ptr(),
		)
	};
	let chooser = native as *mut gtk4_sys::GtkFileChooser;

	apply_file_chooser_defaults(chooser, p, false, multiple);

	let response = run_native_dialog(native as *mut gtk4_sys::GtkNativeDialog);
	if response != gtk4_sys::GTK_RESPONSE_ACCEPT {
		unsafe { g_object_unref(native as *mut _) };
		return None;
	}

	let result = if multiple {
		let model = unsafe { gtk4_sys::gtk_file_chooser_get_files(chooser) };
		collect_file_model(model as *mut GListModel)
	} else {
		let file = unsafe { gtk4_sys::gtk_file_chooser_get_file(chooser) };
		gfile_to_path_buf(file as *mut GFile).into_iter().collect()
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
		gtk4_sys::gtk_file_chooser_native_new(
			title.as_ptr(),
			ptr::null_mut(),
			gtk4_sys::GTK_FILE_CHOOSER_ACTION_SAVE,
			accept.as_ptr(),
			cancel.as_ptr(),
		)
	};
	let chooser = native as *mut gtk4_sys::GtkFileChooser;

	apply_file_chooser_defaults(chooser, p, true, false);

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
