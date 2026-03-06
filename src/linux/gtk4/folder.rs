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
		gtk4_sys::gtk_file_chooser_set_select_multiple(chooser, multiple as i32);
		if let Some(directory) = p.directory {
			let c_path = cstring(directory.to_string_lossy().as_ref());
			let file = gtk4_gio_sys::g_file_new_for_path(c_path.as_ptr());
			gtk4_sys::gtk_file_chooser_set_current_folder(chooser, file, ptr::null_mut());
			g_object_unref(file as *mut _);
		}
	}

	run_native_dialog_f(native as *mut gtk4_sys::GtkNativeDialog, |response| {
		if response != gtk4_sys::GTK_RESPONSE_ACCEPT {
			return None;
		}

		let result = if multiple {
			let model = unsafe { gtk4_sys::gtk_file_chooser_get_files(chooser) };
			collect_file_model(model as *mut GListModel)
		}
		else {
			let file = unsafe { gtk4_sys::gtk_file_chooser_get_file(chooser) };
			gfile_to_path_buf(file as *mut GFile).into_iter().collect()
		};

		Some(result)
	})
}
