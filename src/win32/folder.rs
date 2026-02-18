use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, MAX_PATH, WPARAM};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{
	SHBrowseForFolderW, SHGetPathFromIDListW, BFFM_INITIALIZED, BFFM_SETSELECTIONW,
	BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

use super::*;

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	let title = utf16_null_terminated(p.title);
	let initial_directory = p.directory.and_then(|path| {
		if path.as_os_str().is_empty() {
			None
		} else {
			Some(utf16_null_terminated(&path.to_string_lossy()))
		}
	});

	let mut display_name = vec![0u16; (MAX_PATH as usize) + 1];
	let mut browse_info = BROWSEINFOW::default();
	browse_info.hwndOwner = hwnd(p.owner).unwrap_or_default();
	browse_info.pszDisplayName = PWSTR(display_name.as_mut_ptr());
	browse_info.lpszTitle = PCWSTR(title.as_ptr());
	browse_info.ulFlags = BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE;
	browse_info.lpfn = Some(folder_browse_callback);
	browse_info.lParam = LPARAM(
		initial_directory
			.as_ref()
			.map_or(0, |path| path.as_ptr() as isize),
	);

	let item_list = unsafe { SHBrowseForFolderW(&mut browse_info) };
	if item_list.is_null() {
		return None;
	}

	let mut path = [0u16; MAX_PATH as usize];
	let ok = unsafe { SHGetPathFromIDListW(item_list, &mut path).as_bool() };
	unsafe {
		CoTaskMemFree(Some(item_list as _));
	}

	if !ok {
		return None;
	}

	let length = path.iter().position(|value| *value == 0).unwrap_or(path.len());
	if length == 0 {
		return None;
	}

	Some(PathBuf::from(String::from_utf16_lossy(&path[..length])))
}

unsafe extern "system" fn folder_browse_callback(
	hwnd: HWND,
	message: u32,
	_lparam: LPARAM,
	data: LPARAM,
) -> i32 {
	if message == BFFM_INITIALIZED {
		let initial_path = data.0 as *const u16;
		if !initial_path.is_null() {
			let _ = SendMessageW(
				hwnd,
				BFFM_SETSELECTIONW,
				Some(WPARAM(1)),
				Some(LPARAM(initial_path as isize)),
			);
		}
	}
	0
}
