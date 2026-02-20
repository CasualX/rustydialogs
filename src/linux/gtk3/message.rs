use super::*;

pub fn show(p: &MessageBox<'_>) -> Option<MessageResult> {
	ensure_gtk_initialized();

	let msg_type = match p.icon {
		MessageIcon::Info => gtk_sys::GTK_MESSAGE_INFO,
		MessageIcon::Warning => gtk_sys::GTK_MESSAGE_WARNING,
		MessageIcon::Error => gtk_sys::GTK_MESSAGE_ERROR,
		MessageIcon::Question => gtk_sys::GTK_MESSAGE_QUESTION,
	};

	let title = cstring(p.title);
	let message = cstring(p.message);

	let dialog = unsafe {
		gtk_sys::gtk_message_dialog_new(
			ptr::null_mut(),
			gtk_sys::GTK_DIALOG_MODAL,
			msg_type,
			gtk_sys::GTK_BUTTONS_NONE,
			c"%s".as_ptr(),
			message.as_ptr(),
		)
	};
	unsafe {
		gtk_sys::gtk_window_set_title(dialog as *mut gtk_sys::GtkWindow, title.as_ptr());
	}

	let ok_label = c"OK";
	let cancel_label = c"Cancel";
	let yes_label = c"Yes";
	let no_label = c"No";

	unsafe {
		match p.buttons {
			MessageButtons::Ok => {
				gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, ok_label.as_ptr(), gtk_sys::GTK_RESPONSE_OK);
			}
			MessageButtons::OkCancel => {
				gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, cancel_label.as_ptr(), gtk_sys::GTK_RESPONSE_CANCEL);
				gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, ok_label.as_ptr(), gtk_sys::GTK_RESPONSE_OK);
			}
			MessageButtons::YesNo => {
				gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, no_label.as_ptr(), gtk_sys::GTK_RESPONSE_NO);
				gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, yes_label.as_ptr(), gtk_sys::GTK_RESPONSE_YES);
			}
			MessageButtons::YesNoCancel => {
				gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, cancel_label.as_ptr(), gtk_sys::GTK_RESPONSE_CANCEL);
				gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, no_label.as_ptr(), gtk_sys::GTK_RESPONSE_NO);
				gtk_sys::gtk_dialog_add_button(dialog as *mut gtk_sys::GtkDialog, yes_label.as_ptr(), gtk_sys::GTK_RESPONSE_YES);
			}
		}
	}

	let response = run_dialog(dialog);

	match response {
		gtk_sys::GTK_RESPONSE_OK => Some(MessageResult::Ok),
		gtk_sys::GTK_RESPONSE_CANCEL => Some(MessageResult::Cancel),
		gtk_sys::GTK_RESPONSE_YES => Some(MessageResult::Yes),
		gtk_sys::GTK_RESPONSE_NO => Some(MessageResult::No),
		_ => None,
	}
}
