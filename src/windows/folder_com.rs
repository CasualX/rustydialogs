use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{
	CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
	COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};
use windows::Win32::UI::Shell::{
	FileOpenDialog, IFileDialog, IFileOpenDialog, IShellItem, FOS_FORCEFILESYSTEM,
	FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
};

use super::*;

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	let _com = ComApartment::init()?;

	let dialog: IFileOpenDialog = unsafe {
		CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?
	};
	let file_dialog: IFileDialog = dialog.cast().ok()?;

	let title = utf16cs(p.title);
	unsafe { file_dialog.SetTitle(PCWSTR(title.as_ptr())) }.ok()?;

	let mut options = unsafe { file_dialog.GetOptions() }.ok()?;
	options |= FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST;
	unsafe { file_dialog.SetOptions(options) }.ok()?;

	let directory = utils::abspath(p.directory);
	if directory.is_dir() {
		let path = utf16cs(&directory.to_string_lossy());
		if let Some(folder) = shell_item_from_path(&path) {
			unsafe { file_dialog.SetDefaultFolder(&folder) }.ok()?;
		}
	}

	if unsafe { file_dialog.Show(hwnd(p.owner)) }.is_err() {
		return None;
	}

	let item = unsafe { dialog.GetResult() }.ok()?;
	path_from_shell_item(&item)
}

fn path_from_shell_item(item: &IShellItem) -> Option<PathBuf> {
	let display_name = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) }.ok()?;
	let path = unsafe {
		let ptr = display_name.0;
		if ptr.is_null() {
			None
		} else {
			let mut length = 0usize;
			while *ptr.add(length) != 0 {
				length += 1;
			}
			Some(PathBuf::from(String::from_utf16_lossy(std::slice::from_raw_parts(ptr, length))))
		}
	};
	unsafe {
		CoTaskMemFree(Some(display_name.0 as _));
	}
	path
}

fn shell_item_from_path(path: &[u16]) -> Option<IShellItem> {
	unsafe { SHCreateItemFromParsingName(PCWSTR(path.as_ptr()), None).ok() }
}

struct ComApartment {
	should_uninitialize: bool,
}

impl ComApartment {
	fn init() -> Option<Self> {
		let result = unsafe {
			CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
		};

		if result.is_ok() {
			Some(Self {
				should_uninitialize: true,
			})
		} else if result == RPC_E_CHANGED_MODE {
			Some(Self {
				should_uninitialize: false,
			})
		} else {
			None
		}
	}
}

impl Drop for ComApartment {
	fn drop(&mut self) {
		if self.should_uninitialize {
			unsafe {
				CoUninitialize();
			}
		}
	}
}