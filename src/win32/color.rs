use super::*;
use windows::Win32::Foundation::COLORREF;
use windows::Win32::UI::Controls::Dialogs::{
	ChooseColorW, CHOOSECOLORW, CC_FULLOPEN, CC_RGBINIT,
};

pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	let mut custom_colors = [COLORREF(0); 16];
	let initial = COLORREF(color_value_to_colorref(p.value));

	let mut picker = CHOOSECOLORW::default();
	picker.lStructSize = std::mem::size_of::<CHOOSECOLORW>() as u32;
	picker.rgbResult = initial;
	picker.lpCustColors = custom_colors.as_mut_ptr();
	picker.Flags = CC_RGBINIT | CC_FULLOPEN;

	let ok = unsafe { ChooseColorW(&mut picker).as_bool() };
	if !ok {
		return None;
	}

	Some(colorref_to_color_value(picker.rgbResult))
}

fn color_value_to_colorref(color: ColorValue) -> u32 {
	((color.blue as u32) << 16) | ((color.green as u32) << 8) | color.red as u32
}

fn colorref_to_color_value(colorref: COLORREF) -> ColorValue {
	ColorValue {
		red: (colorref.0 & 0xFF) as u8,
		green: ((colorref.0 >> 8) & 0xFF) as u8,
		blue: ((colorref.0 >> 16) & 0xFF) as u8,
	}
}
