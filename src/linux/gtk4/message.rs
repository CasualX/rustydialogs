use super::*;

pub fn show(p: &MessageBox<'_>) -> Option<MessageResult> {
	ensure_gtk_initialized();

	let msg_type = match p.icon {
		MessageIcon::Info => gtk4_sys::GTK_MESSAGE_INFO,
		MessageIcon::Warning => gtk4_sys::GTK_MESSAGE_WARNING,
		MessageIcon::Error => gtk4_sys::GTK_MESSAGE_ERROR,
		MessageIcon::Question => gtk4_sys::GTK_MESSAGE_QUESTION,
	};

	let title = cstring(p.title);
	let message = cstring(p.message);

	let dialog = unsafe {
		gtk4_sys::gtk_message_dialog_new(
			ptr::null_mut(),
			gtk4_sys::GTK_DIALOG_MODAL,
			msg_type,
			gtk4_sys::GTK_BUTTONS_NONE,
			c"%s".as_ptr(),
			message.as_ptr(),
		)
	};
	unsafe {
		gtk4_sys::gtk_window_set_title(dialog as *mut gtk4_sys::GtkWindow, title.as_ptr());
	}

	let ok_label = c"OK";
	let cancel_label = c"Cancel";
	let yes_label = c"Yes";
	let no_label = c"No";

	unsafe {
		match p.buttons {
			MessageButtons::Ok => {
				gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, ok_label.as_ptr(), gtk4_sys::GTK_RESPONSE_OK);
			}
			MessageButtons::OkCancel => {
				gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, cancel_label.as_ptr(), gtk4_sys::GTK_RESPONSE_CANCEL);
				gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, ok_label.as_ptr(), gtk4_sys::GTK_RESPONSE_OK);
			}
			MessageButtons::YesNo => {
				gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, no_label.as_ptr(), gtk4_sys::GTK_RESPONSE_NO);
				gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, yes_label.as_ptr(), gtk4_sys::GTK_RESPONSE_YES);
			}
			MessageButtons::YesNoCancel => {
				gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, cancel_label.as_ptr(), gtk4_sys::GTK_RESPONSE_CANCEL);
				gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, no_label.as_ptr(), gtk4_sys::GTK_RESPONSE_NO);
				gtk4_sys::gtk_dialog_add_button(dialog as *mut gtk4_sys::GtkDialog, yes_label.as_ptr(), gtk4_sys::GTK_RESPONSE_YES);
			}
		}
	}

	let response = run_dialog(dialog as *mut gtk4_sys::GtkDialog);
	unsafe {
		gtk4_sys::gtk_window_destroy(dialog as *mut gtk4_sys::GtkWindow);
	}

	match response {
		gtk4_sys::GTK_RESPONSE_OK => Some(MessageResult::Ok),
		gtk4_sys::GTK_RESPONSE_CANCEL => Some(MessageResult::Cancel),
		gtk4_sys::GTK_RESPONSE_YES => Some(MessageResult::Yes),
		gtk4_sys::GTK_RESPONSE_NO => Some(MessageResult::No),
		_ => None,
	}
}
