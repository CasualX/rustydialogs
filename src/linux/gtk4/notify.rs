use super::*;
use std::ptr;
use std::sync;

static LIBNOTIFY_INITIALIZED: sync::OnceLock<bool> = sync::OnceLock::new();

fn ensure_libnotify_initialized() {
	let ok = *LIBNOTIFY_INITIALIZED.get_or_init(|| {
		unsafe { libnotify_sys::notify_init(c"rustydialogs".as_ptr()) != 0 }
	});

	if !ok {
		panic!("Failed to initialize libnotify. Ensure a notification service is available.");
	}
}

pub fn notify_popup(p: &NotifyPopup<'_>) {
	ensure_libnotify_initialized();

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
		gtk4_gobject_sys::g_object_unref(notification as *mut _);
	}
}
