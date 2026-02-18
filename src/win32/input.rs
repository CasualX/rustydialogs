use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
	DialogBoxIndirectParamW, EndDialog, GetClientRect, GetDlgItem, GetDlgItemTextW,
	GetWindowLongPtrW, GetWindowRect, MINMAXINFO, MoveWindow, SendMessageW, SetDlgItemTextW, SetWindowLongPtrW,
	BS_DEFPUSHBUTTON, BS_PUSHBUTTON, DLGTEMPLATE, DS_CENTER,
	ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_LEFT, ES_MULTILINE, ES_PASSWORD, ES_WANTRETURN,
	GWLP_USERDATA, IDCANCEL, IDOK, WM_COMMAND, WM_GETMINMAXINFO, WM_INITDIALOG, WM_SIZE,
	WS_BORDER, WS_CAPTION, WS_CHILD, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_TABSTOP,
	WS_THICKFRAME, WS_VISIBLE, WS_VSCROLL,
};

use super::*;

const LABEL_ID: i32 = 1001;
const EDIT_ID: i32 = 1002;

// Minimum dialog client-area size in pixels
const MIN_WIDTH: i32 = 300;
const MIN_HEIGHT_SINGLE: i32 = 120;
const MIN_HEIGHT_MULTI: i32 = 160;

// Margins / layout constants in pixels (used at runtime for anchoring)
const MARGIN: i32 = 12;
const BUTTON_W: i32 = 88;
const BUTTON_H: i32 = 26;
const LABEL_H: i32 = 16;
const EDIT_H_SINGLE: i32 = 22;
const BUTTON_GAP: i32 = 8;

struct InputDialogState {
	message: Vec<u16>,
	initial: Vec<u16>,
	multiline: bool,
	// For single-line mode: the locked window height (full window, pixels) captured after initial layout.
	fixed_height: i32,
	result: Option<String>,
}

pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	let title = utf16_null_terminated(p.title);
	let multiline = matches!(p.mode, TextInputMode::MultiLine);
	let password = matches!(p.mode, TextInputMode::Password);
	let mut template = build_input_dialog_template(&title, multiline, password);
	let mut state = InputDialogState {
		message: utf16_null_terminated(p.message),
		initial: utf16_null_terminated(p.value),
		multiline,
		fixed_height: 0,
		result: None,
	};

	let hinstance = match unsafe { GetModuleHandleW(PCWSTR::null()) } {
		Ok(handle) => handle,
		Err(_) => return None,
	};

	let dialog_result = unsafe {
		DialogBoxIndirectParamW(
			Some(hinstance.into()),
			template.as_mut_ptr().cast::<DLGTEMPLATE>(),
			hwnd(p.owner),
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

/// Re-layout all controls to fill the client area, anchoring the edit box to
/// all four sides and the buttons to the bottom-right corner.
unsafe fn layout_controls(hwnd: HWND, multiline: bool) {
	let mut rc = RECT::default();
	let _ = GetClientRect(hwnd, &mut rc);
	let w = rc.right - rc.left;
	let h = rc.bottom - rc.top;

	// Label — anchored top-left/right
	let label_hwnd = GetDlgItem(Some(hwnd), LABEL_ID);
	if let Ok(lw) = label_hwnd {
		let _ = MoveWindow(
			lw,
			MARGIN,
			MARGIN,
			w - 2 * MARGIN,
			LABEL_H,
			true,
		);
	}

	// Buttons — anchored bottom-right
	let btn_y = h - MARGIN - BUTTON_H;
	let cancel_x = w - MARGIN - BUTTON_W;
	let ok_x = cancel_x - BUTTON_GAP - BUTTON_W;

	let ok_hwnd = GetDlgItem(Some(hwnd), IDOK.0);
	if let Ok(ow) = ok_hwnd {
		let _ = MoveWindow(ow, ok_x, btn_y, BUTTON_W, BUTTON_H, true);
	}

	let cancel_hwnd = GetDlgItem(Some(hwnd), IDCANCEL.0);
	if let Ok(cw) = cancel_hwnd {
		let _ = MoveWindow(cw, cancel_x, btn_y, BUTTON_W, BUTTON_H, true);
	}

	// Edit box — anchored to all four sides
	let edit_top = MARGIN + LABEL_H + 6;
	let edit_hwnd = GetDlgItem(Some(hwnd), EDIT_ID);
	if let Ok(ew) = edit_hwnd {
		let edit_h = if multiline {
			btn_y - edit_top - MARGIN
		} else {
			EDIT_H_SINGLE
		};
		let _ = MoveWindow(
			ew,
			MARGIN,
			edit_top,
			w - 2 * MARGIN,
			edit_h.max(EDIT_H_SINGLE),
			true,
		);
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
				// Move caret to end of initial text
				let edit_hwnd = GetDlgItem(Some(hwnd), EDIT_ID);
				if let Ok(ew) = edit_hwnd {
					SendMessageW(ew, 0x00B1 /*EM_SETSEL*/, Some(WPARAM(usize::MAX)), Some(LPARAM(-1)));
				}
				layout_controls(hwnd, (*state_ptr).multiline);
				// For single-line mode, snapshot the window height so we can lock it.
				if !(*state_ptr).multiline {
					let mut wr = RECT::default();
					let _ = GetWindowRect(hwnd, &mut wr);
					(*state_ptr).fixed_height = wr.bottom - wr.top;
				}
			}
			1
		}
		WM_SIZE => {
			let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut InputDialogState;
			if !state_ptr.is_null() {
				layout_controls(hwnd, (*state_ptr).multiline);
			}
			0
		}
		WM_GETMINMAXINFO => {
			let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut InputDialogState;
			let mmi = lparam.0 as *mut MINMAXINFO;
			if !mmi.is_null() {
				let multiline = !state_ptr.is_null() && (*state_ptr).multiline;
				let min_h = if multiline { MIN_HEIGHT_MULTI } else { MIN_HEIGHT_SINGLE };
				(*mmi).ptMinTrackSize.x = MIN_WIDTH;
				(*mmi).ptMinTrackSize.y = min_h;
				// Single-line: lock height to exactly the initial window height
				if !multiline {
					let fixed_h = if !state_ptr.is_null() && (*state_ptr).fixed_height > 0 {
						(*state_ptr).fixed_height
					} else {
						min_h
					};
					(*mmi).ptMinTrackSize.y = fixed_h;
					(*mmi).ptMaxTrackSize.y = fixed_h;
				}
			}
			0
		}
		WM_COMMAND => {
			let command_id = (wparam.0 & 0xFFFF) as i32;
			if command_id == IDOK.0 {
				let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut InputDialogState;
				if !state_ptr.is_null() {
					let mut buffer = [0u16; 32768];
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
	// Resizable window: WS_THICKFRAME replaces DS_MODALFRAME; DS_CENTER centres on screen.
	// DS_SETFONT lets us specify a font (we use the system default 9pt "MS Shell Dlg 2").
	let dialog_style = (WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_VISIBLE).0 as u32
		| DS_CENTER as u32
		| 0x40u32; // DS_SETFONT

	let edit_style = (WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER).0
		| if multiline {
			(ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32
				| WS_VSCROLL.0
		} else {
			(ES_LEFT | ES_AUTOHSCROLL) as u32
		}
		| if password { ES_PASSWORD as u32 } else { 0 };

	let ok_button_style = (WS_CHILD | WS_VISIBLE | WS_TABSTOP).0 as u32 | BS_DEFPUSHBUTTON as u32;
	let cancel_button_style = (WS_CHILD | WS_VISIBLE | WS_TABSTOP).0 as u32 | BS_PUSHBUTTON as u32;

	// Initial dialog size in dialog units (4x8 DU per char approx).
	// We pick a comfortable default; the user can resize freely.
	// Single-line height is tightly fitted: label + edit + buttons + margins.
	let dlg_w: i16 = 240;
	let dlg_h: i16 = if multiline { 140 } else { 60 };

	let mut data = Vec::with_capacity(512);
	push_u32(&mut data, dialog_style);
	push_u32(&mut data, 0); // extended style
	push_u16(&mut data, 4); // number of items
	push_i16(&mut data, 10); // x
	push_i16(&mut data, 10); // y
	push_i16(&mut data, dlg_w);
	push_i16(&mut data, dlg_h);
	push_u16(&mut data, 0); // menu
	push_u16(&mut data, 0); // window class
	push_utf16z(&mut data, title);

	// DS_SETFONT: point size + typeface name
	push_u16(&mut data, 9); // point size
	push_utf16z(&mut data, &utf16_null_terminated("MS Shell Dlg 2"));

	// Placeholder positions — overwritten immediately in WM_INITDIALOG via layout_controls
	add_dialog_item(
		&mut data,
		(WS_CHILD | WS_VISIBLE).0 as u32,
		0,
		0, 0, 10, 10,
		LABEL_ID as u16,
		0x0082, // STATIC
		&[],
	);

	add_dialog_item(
		&mut data,
		edit_style,
		0,
		0, 0, 10, 10,
		EDIT_ID as u16,
		0x0081, // EDIT
		&[],
	);

	add_dialog_item(
		&mut data,
		ok_button_style,
		0,
		0, 0, 10, 10,
		IDOK.0 as u16,
		0x0080, // BUTTON
		&utf16_null_terminated("OK"),
	);

	add_dialog_item(
		&mut data,
		cancel_button_style,
		0,
		0, 0, 10, 10,
		IDCANCEL.0 as u16,
		0x0080, // BUTTON
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
