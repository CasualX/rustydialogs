use std::ffi::CString;
use std::{ptr, sync};

use super::*;


fn cstring(value: &str) -> CString {
	CString::new(value).unwrap_or_else(|_| CString::new(value.replace('\0', " ")).unwrap())
}

pub fn init(app_id: &str) -> bool {
	static LIBNOTIFY_INITIALIZED: sync::OnceLock<bool> = sync::OnceLock::new();

	// Best effort: libnotify initialization is process-global.
	// Changing app_id after the first initialization may not have any effect.
	let ok = *LIBNOTIFY_INITIALIZED.get_or_init(move || {
		let app_id = cstring(app_id);
		unsafe { libnotify_sys::notify_init(app_id.as_ptr()) != 0 }
	});

	if !ok {
		eprintln!("Failed to initialize libnotify. Ensure a notification service is available.");
	}

	ok
}

pub fn notify(p: &Notification<'_>) {
	if !init(p.app_id) {
		return;
	}

	let (urgency, icon) = match p.icon {
		MessageIcon::Info | MessageIcon::Question => (libnotify_sys::NOTIFY_URGENCY_NORMAL, c"dialog-information"),
		MessageIcon::Warning => (libnotify_sys::NOTIFY_URGENCY_NORMAL, c"dialog-warning"),
		MessageIcon::Error => (libnotify_sys::NOTIFY_URGENCY_CRITICAL, c"dialog-error"),
	};

	let timeout = if p.timeout <= 0 {
		libnotify_sys::NOTIFY_EXPIRES_NEVER
	} else {
		p.timeout
	};

	let title = cstring(p.title);
	let message = cstring(p.message);

	let notification = unsafe {
		libnotify_sys::notify_notification_new(title.as_ptr(), message.as_ptr(), icon.as_ptr())
	};
	if notification.is_null() {
		return;
	}

	unsafe {
		libnotify_sys::notify_notification_set_urgency(notification, urgency);
		libnotify_sys::notify_notification_set_timeout(notification, timeout);
		let _ = libnotify_sys::notify_notification_show(notification, ptr::null_mut());
		gobject_sys::g_object_unref(notification as *mut _);
	}
}
