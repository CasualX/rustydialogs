use super::*;

unsafe fn style_response_button(dialog: *mut gtk4_sys::GtkDialog, response: i32) {
	let button = gtk4_sys::gtk_dialog_get_widget_for_response(dialog, response);
	if button.is_null() {
		return;
	}

	gtk4_sys::gtk_widget_set_size_request(button, 110, -1);
	gtk4_sys::gtk_widget_set_margin_start(button, 6);
	gtk4_sys::gtk_widget_set_margin_end(button, 6);
	gtk4_sys::gtk_widget_set_margin_top(button, 8);
	gtk4_sys::gtk_widget_set_margin_bottom(button, 8);
}

pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	match p.mode {
		TextInputMode::SingleLine => text_input_entry(p, false),
		TextInputMode::MultiLine => text_input_multiline(p),
		TextInputMode::Password => text_input_entry(p, true),
	}
}

fn text_input_entry(p: &TextInput<'_>, password: bool) -> Option<String> {
	ensure_gtk_initialized();

	let dialog = unsafe { gtk4_sys::gtk_dialog_new() };
	let title = cstring(p.title);
	let message = cstring(p.message);
	let value = cstring(p.value);
	let ok = c"OK";
	let cancel = c"Cancel";

	unsafe {
		gtk4_sys::gtk_window_set_title(dialog as *mut gtk4_sys::GtkWindow, title.as_ptr());
		gtk4_sys::gtk_window_set_default_size(dialog as *mut gtk4_sys::GtkWindow, 520, -1);
		gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, cancel.as_ptr(), gtk4_sys::GTK_RESPONSE_CANCEL);
		gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, ok.as_ptr(), gtk4_sys::GTK_RESPONSE_OK);
		style_response_button(dialog as *mut gtk4_sys::GtkDialog, gtk4_sys::GTK_RESPONSE_CANCEL);
		style_response_button(dialog as *mut gtk4_sys::GtkDialog, gtk4_sys::GTK_RESPONSE_OK);

		let content = gtk4_sys::gtk_dialog_get_content_area(dialog as *mut gtk4_sys::GtkDialog);
		gtk4_sys::gtk_box_set_spacing(content as *mut gtk4_sys::GtkBox, 10);
		gtk4_sys::gtk_widget_set_vexpand(content as *mut gtk4_sys::GtkWidget, 1);
		gtk4_sys::gtk_widget_set_hexpand(content as *mut gtk4_sys::GtkWidget, 1);
		gtk4_sys::gtk_widget_set_margin_start(content as *mut gtk4_sys::GtkWidget, 14);
		gtk4_sys::gtk_widget_set_margin_end(content as *mut gtk4_sys::GtkWidget, 14);
		gtk4_sys::gtk_widget_set_margin_top(content as *mut gtk4_sys::GtkWidget, 14);
		gtk4_sys::gtk_widget_set_margin_bottom(content as *mut gtk4_sys::GtkWidget, 14);

		let label = gtk4_sys::gtk_label_new(message.as_ptr());
		gtk4_sys::gtk_label_set_xalign(label as *mut gtk4_sys::GtkLabel, 0.0);
		let entry = gtk4_sys::gtk_entry_new();
		gtk4_sys::gtk_editable_set_text(entry as *mut gtk4_sys::GtkEditable, value.as_ptr());
		if password {
			gtk4_sys::gtk_entry_set_visibility(entry as *mut gtk4_sys::GtkEntry, 0);
		}

		gtk4_sys::gtk_box_append(content as *mut gtk4_sys::GtkBox, label);
		gtk4_sys::gtk_box_append(content as *mut gtk4_sys::GtkBox, entry);

		let response = run_dialog(dialog as *mut gtk4_sys::GtkDialog);
		let result = if response == gtk4_sys::GTK_RESPONSE_OK {
			let text_ptr = gtk4_sys::gtk_editable_get_text(entry as *mut gtk4_sys::GtkEditable);
			if text_ptr.is_null() {
				None
			} else {
				Some(CStr::from_ptr(text_ptr).to_string_lossy().to_string())
			}
		} else {
			None
		};

		gtk4_sys::gtk_window_destroy(dialog as *mut gtk4_sys::GtkWindow);
		result
	}
}

fn text_input_multiline(p: &TextInput<'_>) -> Option<String> {
	ensure_gtk_initialized();

	let dialog = unsafe { gtk4_sys::gtk_dialog_new() };
	let title = cstring(p.title);
	let message = cstring(p.message);
	let value = cstring(p.value);
	let ok = c"OK";
	let cancel = c"Cancel";

	unsafe {
		gtk4_sys::gtk_window_set_title(dialog as *mut gtk4_sys::GtkWindow, title.as_ptr());
		gtk4_sys::gtk_window_set_default_size(dialog as *mut gtk4_sys::GtkWindow, 500, 380);
		gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, cancel.as_ptr(), gtk4_sys::GTK_RESPONSE_CANCEL);
		gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, ok.as_ptr(), gtk4_sys::GTK_RESPONSE_OK);
		style_response_button(dialog as *mut gtk4_sys::GtkDialog, gtk4_sys::GTK_RESPONSE_CANCEL);
		style_response_button(dialog as *mut gtk4_sys::GtkDialog, gtk4_sys::GTK_RESPONSE_OK);

		let content = gtk4_sys::gtk_dialog_get_content_area(dialog as *mut gtk4_sys::GtkDialog);
		gtk4_sys::gtk_box_set_spacing(content as *mut gtk4_sys::GtkBox, 10);
		gtk4_sys::gtk_widget_set_margin_start(content as *mut gtk4_sys::GtkWidget, 14);
		gtk4_sys::gtk_widget_set_margin_end(content as *mut gtk4_sys::GtkWidget, 14);
		gtk4_sys::gtk_widget_set_margin_top(content as *mut gtk4_sys::GtkWidget, 14);
		gtk4_sys::gtk_widget_set_margin_bottom(content as *mut gtk4_sys::GtkWidget, 14);

		let label = gtk4_sys::gtk_label_new(message.as_ptr());
		gtk4_sys::gtk_label_set_xalign(label as *mut gtk4_sys::GtkLabel, 0.0);
		let scrolled = gtk4_sys::gtk_scrolled_window_new();
		let text_view = gtk4_sys::gtk_text_view_new();
		let buffer = gtk4_sys::gtk_text_view_get_buffer(text_view as *mut gtk4_sys::GtkTextView);
		gtk4_sys::gtk_text_buffer_set_text(buffer, value.as_ptr(), -1);

		gtk4_sys::gtk_widget_set_size_request(scrolled, 480, 280);
		gtk4_sys::gtk_widget_set_vexpand(scrolled, 1);
		gtk4_sys::gtk_widget_set_hexpand(scrolled, 1);
		gtk4_sys::gtk_widget_set_vexpand(text_view, 1);
		gtk4_sys::gtk_widget_set_hexpand(text_view, 1);
		gtk4_sys::gtk_scrolled_window_set_child(scrolled as *mut gtk4_sys::GtkScrolledWindow, text_view);

		gtk4_sys::gtk_box_append(content as *mut gtk4_sys::GtkBox, label);
		gtk4_sys::gtk_box_append(content as *mut gtk4_sys::GtkBox, scrolled);

		let response = run_dialog(dialog as *mut gtk4_sys::GtkDialog);
		let result = if response == gtk4_sys::GTK_RESPONSE_OK {
			let mut start = std::mem::zeroed();
			let mut end = std::mem::zeroed();
			gtk4_sys::gtk_text_buffer_get_bounds(buffer, &mut start, &mut end);
			let text_ptr = gtk4_sys::gtk_text_buffer_get_text(buffer, &mut start, &mut end, 0);
			if text_ptr.is_null() {
				None
			} else {
				let text = CStr::from_ptr(text_ptr).to_string_lossy().to_string();
				gtk4_glib_sys::g_free(text_ptr as *mut _);
				Some(text)
			}
		} else {
			None
		};

		gtk4_sys::gtk_window_destroy(dialog as *mut gtk4_sys::GtkWindow);
		result
	}
}
