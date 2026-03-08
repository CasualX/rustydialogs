use super::*;

fn apply_file_chooser_defaults(dialog: *mut gtk4_sys::GtkFileChooser, p: &FileDialog<'_>, save: bool, multiple: bool) {
	unsafe {
		gtk4_sys::gtk_file_chooser_set_select_multiple(dialog, multiple as i32);
		let _ = save;

		if let Some(filters) = p.filters {
			for filter in filters {
				add_filter(dialog, filter);
			}
			add_filter(dialog, &FileFilter::ALL_FILES);
		}
	}

	if let Some(path) = utils::abspath(p.path) {
		apply_initial_path(dialog, path.as_ref());
	}
}

fn apply_initial_path(dialog: *mut gtk4_sys::GtkFileChooser, path: &Path) {
	if path.is_dir() {
		set_current_folder(dialog, path);
		return;
	}

	if path.is_file() {
		set_file(dialog, path);
		return;
	}

	if let Some(parent) = path.parent().filter(|parent| parent.is_dir()) {
		set_current_folder(dialog, parent);
	}

	if let Some(name) = path.file_name() {
		if let Some(c_name) = os_cstring(name) {
			unsafe { gtk4_sys::gtk_file_chooser_set_current_name(dialog, c_name.as_ptr()); }
		}
	}
}

fn set_current_folder(dialog: *mut gtk4_sys::GtkFileChooser, path: &Path) {
	if let Some(c_path) = os_cstring(path.as_os_str()) {
		unsafe {
			let file = gtk4_gio_sys::g_file_new_for_path(c_path.as_ptr());
			gtk4_sys::gtk_file_chooser_set_current_folder(dialog, file, ptr::null_mut());
			g_object_unref(file as *mut _);
		}
	}
}

fn set_file(dialog: *mut gtk4_sys::GtkFileChooser, path: &Path) {
	if let Some(c_path) = os_cstring(path.as_os_str()) {
		unsafe {
			let file = gtk4_gio_sys::g_file_new_for_path(c_path.as_ptr());
			gtk4_sys::gtk_file_chooser_set_file(dialog, file, ptr::null_mut());
			g_object_unref(file as *mut _);
		}
	}
}

fn add_filter(dialog: *mut gtk4_sys::GtkFileChooser, filter: &FileFilter) {
	unsafe {
		let gtk_filter = gtk4_sys::gtk_file_filter_new();
		let name = cstring(filter.name);
		gtk4_sys::gtk_file_filter_set_name(gtk_filter, name.as_ptr());
		for pattern in filter.patterns {
			let pattern = cstring(pattern);
			gtk4_sys::gtk_file_filter_add_pattern(gtk_filter, pattern.as_ptr());
		}
		gtk4_sys::gtk_file_chooser_add_filter(dialog, gtk_filter);
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

	run_native_dialog_f(native as *mut gtk4_sys::GtkNativeDialog, |response| {
		if response != gtk4_sys::GTK_RESPONSE_ACCEPT {
			return None;
		}

		let result = if multiple {
			let model = unsafe { gtk4_sys::gtk_file_chooser_get_files(chooser) };
			collect_file_model(model as *mut GListModel)
		} else {
			let file = unsafe { gtk4_sys::gtk_file_chooser_get_file(chooser) };
			gfile_to_path_buf(file as *mut GFile).into_iter().collect()
		};

		Some(result)
	})
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

	run_native_dialog_f(native as *mut gtk4_sys::GtkNativeDialog, |response| {
		if response != gtk4_sys::GTK_RESPONSE_ACCEPT {
			return None;
		}

		let file = unsafe { gtk4_sys::gtk_file_chooser_get_file(chooser) };
		gfile_to_path_buf(file as *mut GFile)
	})
}
