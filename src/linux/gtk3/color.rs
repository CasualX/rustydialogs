use super::*;

pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	ensure_gtk_initialized();

	let title = cstring(p.title);
	let dialog = unsafe { gtk_sys::gtk_color_chooser_dialog_new(title.as_ptr(), ptr::null_mut()) };

	let mut rgba = gdk_sys::GdkRGBA {
		red: f64::from(p.value.red) / 255.0,
		green: f64::from(p.value.green) / 255.0,
		blue: f64::from(p.value.blue) / 255.0,
		alpha: 1.0,
	};

	unsafe {
		gtk_sys::gtk_color_chooser_set_rgba(dialog as *mut gtk_sys::GtkColorChooser, &rgba);
	}

	let response = unsafe { gtk_sys::gtk_dialog_run(dialog as *mut gtk_sys::GtkDialog) };
	if response != gtk_sys::GTK_RESPONSE_OK {
		unsafe {
			gtk_sys::gtk_widget_destroy(dialog);
			while gtk_sys::gtk_events_pending() != 0 {
				gtk_sys::gtk_main_iteration();
			}
		}
		return None;
	}

	unsafe {
		gtk_sys::gtk_color_chooser_get_rgba(dialog as *mut gtk_sys::GtkColorChooser, &mut rgba);
		gtk_sys::gtk_widget_destroy(dialog);
		while gtk_sys::gtk_events_pending() != 0 {
			gtk_sys::gtk_main_iteration();
		}
	}

	Some(ColorValue {
		red: (rgba.red.clamp(0.0, 1.0) * 255.0).round() as u8,
		green: (rgba.green.clamp(0.0, 1.0) * 255.0).round() as u8,
		blue: (rgba.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
	})
}
