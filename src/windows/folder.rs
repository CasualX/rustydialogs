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
	let title = utf16cs(p.title);

	let mut display_name = [0u16; MAX_PATH as usize];
	let mut browse_info = BROWSEINFOW::default();
	browse_info.hwndOwner = hwnd(p.owner).unwrap_or_default();
	browse_info.pszDisplayName = PWSTR(display_name.as_mut_ptr());
	browse_info.lpszTitle = PCWSTR(title.as_ptr());
	browse_info.ulFlags = BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE;
	browse_info.lpfn = Some(folder_browse_callback);
	browse_info.lParam = LPARAM((p as *const FolderDialog<'_>).expose_provenance() as isize);

	let item_list = unsafe { SHBrowseForFolderW(&mut browse_info) };
	if item_list.is_null() {
		return None;
	}

	let mut path = [0u16; MAX_PATH as usize];
	let ok = unsafe { SHGetPathFromIDListW(item_list, &mut path).as_bool() };
	unsafe {
		CoTaskMemFree(Some(item_list as _));
	}

	if ok {
		let length = path.iter().position(|value| *value == 0).unwrap_or(path.len());
		let result = PathBuf::from(String::from_utf16_lossy(&path[..length]));
		Some(result)
	}
	else {
		None
	}
}

unsafe extern "system" fn folder_browse_callback(hwnd: HWND, message: u32, _lparam: LPARAM, data: LPARAM) -> i32 {
	let p = unsafe { &*std::ptr::with_exposed_provenance::<FolderDialog<'_>>(data.0 as usize) };
	if message == BFFM_INITIALIZED {
		let directory = p.directory.map(|path| utf16cs(&path.to_string_lossy()));
		let lparam = directory.as_ref().map(|dir| LPARAM(dir.as_ptr().expose_provenance() as isize));
		let _ = SendMessageW(hwnd, BFFM_SETSELECTIONW, Some(WPARAM(1)), lparam);
	}
	return 0;
}
