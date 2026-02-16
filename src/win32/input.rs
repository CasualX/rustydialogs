use super::*;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
	DialogBoxIndirectParamW, EndDialog, GetDlgItemTextW, GetWindowLongPtrW, SetDlgItemTextW,
	SetWindowLongPtrW, BS_DEFPUSHBUTTON, BS_PUSHBUTTON, DLGTEMPLATE, DS_MODALFRAME,
	ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_LEFT, ES_MULTILINE, ES_PASSWORD, ES_WANTRETURN,
	GWLP_USERDATA,
	IDCANCEL, IDOK, WM_COMMAND, WM_INITDIALOG, WS_BORDER, WS_CAPTION, WS_CHILD,
	WS_POPUP, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

const LABEL_ID: i32 = 1001;
const EDIT_ID: i32 = 1002;

struct InputDialogState {
	message: Vec<u16>,
	initial: Vec<u16>,
	result: Option<String>,
}

pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	let title = utf16_null_terminated(p.title);
	let mut template = build_input_dialog_template(
		&title,
		matches!(p.mode, TextInputMode::Multi),
		matches!(p.mode, TextInputMode::Password),
	);
	let mut state = InputDialogState {
		message: utf16_null_terminated(p.message),
		initial: utf16_null_terminated(p.value),
		result: None,
	};

	let hinstance = match unsafe { GetModuleHandleW(PCWSTR::null()) } {
		Ok(handle) => handle,
		Err(_) => return None,
	};

	let dialog_result = unsafe {
		DialogBoxIndirectParamW(
			hinstance,
			template.as_mut_ptr().cast::<DLGTEMPLATE>(),
			None,
			Some(input_dialog_proc),
			LPARAM((&mut state as *mut InputDialogState) as isize),
		)
	};

	if dialog_result == IDOK.0 as isize {
		state.result
	} else {
		None
	}
}

unsafe extern "system" fn input_dialog_proc(
	hwnd: HWND,
	msg: u32,
	wparam: WPARAM,
	lparam: LPARAM,
) -> isize {
	match msg {
		WM_INITDIALOG => {
			let state_ptr = lparam.0 as *mut InputDialogState;
			SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
			if !state_ptr.is_null() {
				let _ = SetDlgItemTextW(hwnd, LABEL_ID, PCWSTR((*state_ptr).message.as_ptr()));
				let _ = SetDlgItemTextW(hwnd, EDIT_ID, PCWSTR((*state_ptr).initial.as_ptr()));
			}
			1
		}
		WM_COMMAND => {
			let command_id = (wparam.0 & 0xFFFF) as i32;
			if command_id == IDOK.0 {
				let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut InputDialogState;
				if !state_ptr.is_null() {
					let mut buffer = [0u16; 8192];
					let length = GetDlgItemTextW(hwnd, EDIT_ID, &mut buffer) as usize;
					(*state_ptr).result = Some(String::from_utf16_lossy(&buffer[..length]));
				}
				let _ = EndDialog(hwnd, IDOK.0 as isize);
				return 1;
			}

			if command_id == IDCANCEL.0 {
				let _ = EndDialog(hwnd, IDCANCEL.0 as isize);
				return 1;
			}

			0
		}
		_ => 0,
	}
}

fn build_input_dialog_template(title: &[u16], multiline: bool, password: bool) -> Vec<u8> {
	let dialog_style = (WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE).0 as u32 | DS_MODALFRAME as u32;
	let edit_style = (WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER).0
		| if multiline {
			(ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32
		} else {
			(ES_LEFT | ES_AUTOHSCROLL) as u32
		}
		| if password { ES_PASSWORD as u32 } else { 0 };

	let ok_button_style = (WS_CHILD | WS_VISIBLE | WS_TABSTOP).0 as u32
		| if multiline {
			BS_PUSHBUTTON as u32
		} else {
			BS_DEFPUSHBUTTON as u32
		};

	let mut data = Vec::with_capacity(512);
	push_u32(&mut data, dialog_style);
	push_u32(&mut data, 0);
	push_u16(&mut data, 4);
	push_i16(&mut data, 10);
	push_i16(&mut data, 10);
	push_i16(&mut data, 220);
	push_i16(&mut data, if multiline { 125 } else { 92 });
	push_u16(&mut data, 0);
	push_u16(&mut data, 0);
	push_utf16z(&mut data, title);

	add_dialog_item(
		&mut data,
		(WS_CHILD | WS_VISIBLE).0 as u32,
		0,
		8,
		8,
		204,
		16,
		LABEL_ID as u16,
		0x0082,
		&[],
	);

	add_dialog_item(
		&mut data,
		edit_style,
		0,
		8,
		26,
		204,
		if multiline { 52 } else { 14 },
		EDIT_ID as u16,
		0x0081,
		&[],
	);

	add_dialog_item(
		&mut data,
		ok_button_style,
		0,
		84,
		if multiline { 86 } else { 48 },
		58,
		14,
		IDOK.0 as u16,
		0x0080,
		&utf16_null_terminated("OK"),
	);

	add_dialog_item(
		&mut data,
		(WS_CHILD | WS_VISIBLE | WS_TABSTOP).0 as u32 | BS_PUSHBUTTON as u32,
		0,
		150,
		if multiline { 86 } else { 48 },
		58,
		14,
		IDCANCEL.0 as u16,
		0x0080,
		&utf16_null_terminated("Cancel"),
	);

	data
}

#[allow(clippy::too_many_arguments)]
fn add_dialog_item(
	data: &mut Vec<u8>,
	style: u32,
	extended_style: u32,
	x: i16,
	y: i16,
	cx: i16,
	cy: i16,
	id: u16,
	class_ordinal: u16,
	caption: &[u16],
) {
	align_dword(data);
	push_u32(data, style);
	push_u32(data, extended_style);
	push_i16(data, x);
	push_i16(data, y);
	push_i16(data, cx);
	push_i16(data, cy);
	push_u16(data, id);
	push_u16(data, 0xFFFF);
	push_u16(data, class_ordinal);
	if caption.is_empty() {
		push_u16(data, 0);
	} else {
		push_utf16z(data, caption);
	}
	push_u16(data, 0);
}

fn align_dword(data: &mut Vec<u8>) {
	while data.len() % 4 != 0 {
		data.push(0);
	}
}

fn push_u16(data: &mut Vec<u8>, value: u16) {
	data.extend_from_slice(&value.to_le_bytes());
}

fn push_i16(data: &mut Vec<u8>, value: i16) {
	data.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(data: &mut Vec<u8>, value: u32) {
	data.extend_from_slice(&value.to_le_bytes());
}

fn push_utf16z(data: &mut Vec<u8>, value: &[u16]) {
	if value.is_empty() {
		push_u16(data, 0);
		return;
	}

	for ch in value.iter().copied() {
		push_u16(data, ch);
	}
	if *value.last().unwrap_or(&1) != 0 {
		push_u16(data, 0);
	}
}
