use super::*;

pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	match p.mode {
		TextInputMode::SingleLine => text_input_entry(p, false),
		TextInputMode::MultiLine => text_input_multiline(p),
		TextInputMode::Password => text_input_entry(p, true),
	}
}

fn text_input_entry(p: &TextInput<'_>, password: bool) -> Option<String> {
	ensure_gtk_initialized();

	let dialog = unsafe { gtk_sys::gtk_dialog_new() };
	let title = cstring(p.title);
	let message = cstring(p.message);
	let value = cstring(p.value);
	let ok = c"OK";
	let cancel = c"Cancel";

	unsafe {
		gtk_sys::gtk_window_set_title(dialog as *mut gtk_sys::GtkWindow, title.as_ptr());
		gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, cancel.as_ptr(), gtk_sys::GTK_RESPONSE_CANCEL);
		gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, ok.as_ptr(), gtk_sys::GTK_RESPONSE_OK);

		let content = gtk_sys::gtk_dialog_get_content_area(dialog as *mut gtk_sys::GtkDialog);
		let label = gtk_sys::gtk_label_new(message.as_ptr());
		let entry = gtk_sys::gtk_entry_new();
		gtk_sys::gtk_entry_set_text(entry as *mut gtk_sys::GtkEntry, value.as_ptr());
		if password {
			gtk_sys::gtk_entry_set_visibility(entry as *mut gtk_sys::GtkEntry, 0);
		}

		gtk_sys::gtk_box_pack_start(content as *mut gtk_sys::GtkBox, label, 0, 0, 6);
		gtk_sys::gtk_box_pack_start(content as *mut gtk_sys::GtkBox, entry, 0, 0, 6);
		gtk_sys::gtk_widget_show_all(dialog);

		let response = gtk_sys::gtk_dialog_run(dialog as *mut gtk_sys::GtkDialog);
		let result = if response == gtk_sys::GTK_RESPONSE_OK {
			let text_ptr = gtk_sys::gtk_entry_get_text(entry as *mut gtk_sys::GtkEntry);
			if text_ptr.is_null() {
				None
			} else {
				Some(CStr::from_ptr(text_ptr).to_string_lossy().to_string())
			}
		} else {
			None
		};

		gtk_sys::gtk_widget_destroy(dialog);
		while gtk_sys::gtk_events_pending() != 0 {
			gtk_sys::gtk_main_iteration();
		}

		result
	}
}

fn text_input_multiline(p: &TextInput<'_>) -> Option<String> {
	ensure_gtk_initialized();

	let dialog = unsafe { gtk_sys::gtk_dialog_new() };
	let title = cstring(p.title);
	let message = cstring(p.message);
	let value = cstring(p.value);
	let ok = c"OK";
	let cancel = c"Cancel";

	unsafe {
		gtk_sys::gtk_window_set_title(dialog as *mut gtk_sys::GtkWindow, title.as_ptr());
		gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, cancel.as_ptr(), gtk_sys::GTK_RESPONSE_CANCEL);
		gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, ok.as_ptr(), gtk_sys::GTK_RESPONSE_OK);

		let content = gtk_sys::gtk_dialog_get_content_area(dialog as *mut gtk_sys::GtkDialog);
		let label = gtk_sys::gtk_label_new(message.as_ptr());
		let scrolled = gtk_sys::gtk_scrolled_window_new(ptr::null_mut(), ptr::null_mut());
		let text_view = gtk_sys::gtk_text_view_new();
		let buffer = gtk_sys::gtk_text_view_get_buffer(text_view as *mut gtk_sys::GtkTextView);
		gtk_sys::gtk_text_buffer_set_text(buffer, value.as_ptr(), -1);

		gtk_sys::gtk_widget_set_size_request(scrolled, 480, 280);
		gtk_sys::gtk_container_add(scrolled as *mut gtk_sys::GtkContainer, text_view);

		gtk_sys::gtk_box_pack_start(content as *mut gtk_sys::GtkBox, label, 0, 0, 6);
		gtk_sys::gtk_box_pack_start(content as *mut gtk_sys::GtkBox, scrolled, 1, 1, 6);
		gtk_sys::gtk_widget_show_all(dialog);

		let response = gtk_sys::gtk_dialog_run(dialog as *mut gtk_sys::GtkDialog);
		let result = if response == gtk_sys::GTK_RESPONSE_OK {
			let mut start = std::mem::zeroed();
			let mut end = std::mem::zeroed();
			gtk_sys::gtk_text_buffer_get_bounds(buffer, &mut start, &mut end);
			let text_ptr = gtk_sys::gtk_text_buffer_get_text(buffer, &mut start, &mut end, 0);
			if text_ptr.is_null() {
				None
			} else {
				let text = CStr::from_ptr(text_ptr).to_string_lossy().to_string();
				glib_sys::g_free(text_ptr as *mut _);
				Some(text)
			}
		} else {
			None
		};

		gtk_sys::gtk_widget_destroy(dialog);
		while gtk_sys::gtk_events_pending() != 0 {
			gtk_sys::gtk_main_iteration();
		}

		result
	}
}
