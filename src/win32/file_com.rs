use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{
	CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
	COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};
use windows::Win32::UI::Shell::{
	FileOpenDialog, FileSaveDialog, IFileDialog, IFileOpenDialog, IFileSaveDialog, IShellItem,
	FOS_ALLOWMULTISELECT, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_OVERWRITEPROMPT,
	FOS_PATHMUSTEXIST, SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
};
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;

use super::*;

pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	pick_files_impl(p, false).and_then(|paths| paths.into_iter().next())
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	pick_files_impl(p, true)
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let _com = ComApartment::init()?;

	let dialog: IFileSaveDialog = unsafe {
		CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER).ok()?
	};
	let file_dialog: IFileDialog = dialog.cast().ok()?;

	configure_dialog(&file_dialog, p, false, true)?;

	if unsafe { file_dialog.Show(hwnd(p.owner)) }.is_err() {
		return None;
	}

	let item = unsafe { dialog.GetResult() }.ok()?;
	path_from_shell_item(&item)
}

fn pick_files_impl(p: &FileDialog<'_>, allow_multiple_selects: bool) -> Option<Vec<PathBuf>> {
	let _com = ComApartment::init()?;

	let dialog: IFileOpenDialog = unsafe {
		CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?
	};
	let file_dialog: IFileDialog = dialog.cast().ok()?;

	configure_dialog(&file_dialog, p, allow_multiple_selects, false)?;

	if unsafe { file_dialog.Show(hwnd(p.owner)) }.is_err() {
		return None;
	}

	if allow_multiple_selects {
		let items = unsafe { dialog.GetResults() }.ok()?;
		let count = unsafe { items.GetCount() }.ok()?;
		let mut paths = Vec::with_capacity(count as usize);
		for index in 0..count {
			let item = unsafe { items.GetItemAt(index) }.ok()?;
			paths.push(path_from_shell_item(&item)?);
		}
		Some(paths)
	} else {
		let item = unsafe { dialog.GetResult() }.ok()?;
		Some(vec![path_from_shell_item(&item)?])
	}
}

fn configure_dialog(dialog: &IFileDialog, p: &FileDialog<'_>, allow_multiple_selects: bool, is_save: bool) -> Option<()> {
	let title = utf16cs(p.title);
	let mut filters = build_windows_filter(p.filter);

	if !p.title.is_empty() {
		unsafe { dialog.SetTitle(PCWSTR(title.as_ptr())) }.ok()?;
	}

	if let Some(filters) = filters.as_mut() {
		unsafe { dialog.SetFileTypes(&filters.specs) }.ok()?;
	}

	let mut options = unsafe { dialog.GetOptions() }.ok()?;
	options |= FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST;
	if is_save {
		options |= FOS_OVERWRITEPROMPT;
	} else if allow_multiple_selects {
		options |= FOS_ALLOWMULTISELECT | FOS_FILEMUSTEXIST;
	} else {
		options |= FOS_FILEMUSTEXIST;
	}
	unsafe { dialog.SetOptions(options) }.ok()?;

	let initial_path = utils::abspath(p.path);
	if initial_path.is_dir() {
		let path = utf16cs(&initial_path.to_string_lossy());
		if let Some(folder) = shell_item_from_path(&path) {
			unsafe { dialog.SetDefaultFolder(&folder) }.ok()?;
		}
	} else {
		if let Some(parent) = initial_path.parent() {
			let parent_path = utf16cs(&parent.to_string_lossy());
			if let Some(folder) = shell_item_from_path(&parent_path) {
				unsafe { dialog.SetDefaultFolder(&folder) }.ok()?;
			}
		}
		if let Some(file_name) = initial_path.file_name().and_then(|name| name.to_str()) {
			let name = utf16cs(file_name);
			unsafe { dialog.SetFileName(PCWSTR(name.as_ptr())) }.ok()?;
		}
	}

	Some(())
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

struct DialogFilters {
	_names: Vec<Vec<u16>>,
	_specs_storage: Vec<Vec<u16>>,
	specs: Vec<COMDLG_FILTERSPEC>,
}

fn build_windows_filter(filter: Option<&[FileFilter<'_>]>) -> Option<DialogFilters> {
	let filter = filter?;
	if filter.is_empty() {
		return None;
	}

	let mut names = Vec::with_capacity(filter.len() + 1);
	let mut specs_storage = Vec::with_capacity(filter.len() + 1);

	for entry in filter {
		names.push(utf16cs(entry.desc));
		specs_storage.push(utf16cs(&utils::PrintJoin {
			parts: entry.patterns,
			separator: ";",
		}.to_string()));
	}

	names.push(utf16cs("All Files"));
	specs_storage.push(utf16cs("*.*"));

	let mut specs = Vec::with_capacity(names.len());
	for index in 0..names.len() {
		specs.push(COMDLG_FILTERSPEC {
			pszName: PCWSTR(names[index].as_ptr()),
			pszSpec: PCWSTR(specs_storage[index].as_ptr()),
		});
	}

	Some(DialogFilters {
		_names: names,
		_specs_storage: specs_storage,
		specs,
	})
}

struct ComApartment {
	should_uninitialize: bool,
}

impl ComApartment {
	fn init() -> Option<Self> {
		let result = unsafe {
			CoInitializeEx(
				None,
				COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
			)
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
