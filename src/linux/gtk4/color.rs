use super::*;

pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	ensure_gtk_initialized();

	let title = cstring(p.title);
	let dialog = unsafe { gtk4_sys::gtk_color_chooser_dialog_new(title.as_ptr(), ptr::null_mut()) };

	let mut rgba = gdk4_sys::GdkRGBA {
		red: f32::from(p.value.red) / 255.0,
		green: f32::from(p.value.green) / 255.0,
		blue: f32::from(p.value.blue) / 255.0,
		alpha: 1.0,
	};

	unsafe {
		gtk4_sys::gtk_color_chooser_set_rgba(dialog as *mut gtk4_sys::GtkColorChooser, &rgba);
	}

	run_dialog_f(dialog as *mut gtk4_sys::GtkDialog, |response| {
		if response != gtk4_sys::GTK_RESPONSE_OK {
			return None;
		}

		unsafe {
			gtk4_sys::gtk_color_chooser_get_rgba(dialog as *mut gtk4_sys::GtkColorChooser, &mut rgba);
		}

		Some(ColorValue {
			red: (rgba.red.clamp(0.0, 1.0) * 255.0).round() as u8,
			green: (rgba.green.clamp(0.0, 1.0) * 255.0).round() as u8,
			blue: (rgba.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
		})
	})
}
