use windows::core::{Interface, PCWSTR};
use windows::Win32::System::Com::{
	CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER,
};
use windows::Win32::UI::Shell::{
	FileOpenDialog, FileSaveDialog, IFileDialog, IFileOpenDialog, IFileSaveDialog, IShellItem,
	IShellItemArray, SHCreateItemFromParsingName, SIGDN_FILESYSPATH, FILEOPENDIALOGOPTIONS,
	FOS_ALLOWMULTISELECT, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_OVERWRITEPROMPT,
	FOS_PATHMUSTEXIST, FOS_PICKFOLDERS,
};
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;

use super::*;

#[allow(dead_code)]
pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let _com = com::Apartment::init().ok()?;
	let file_open_dialog: IFileOpenDialog = unsafe {
		CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?
	};
	let file_dialog: IFileDialog = file_open_dialog.cast().ok()?;
	show_dialog(&file_dialog, p.title, p.path, p.filter, p.owner, FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_FILEMUSTEXIST)?;
	let item = unsafe { file_open_dialog.GetResult() }.ok()?;
	path_from_shell_item(&item)
}

#[allow(dead_code)]
pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	let _com = com::Apartment::init().ok()?;
	let file_open_dialog: IFileOpenDialog = unsafe {
		CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?
	};
	let file_dialog: IFileDialog = file_open_dialog.cast().ok()?;
	show_dialog(&file_dialog, p.title, p.path, p.filter, p.owner, FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_FILEMUSTEXIST | FOS_ALLOWMULTISELECT)?;
	let items = unsafe { file_open_dialog.GetResults() }.ok()?;
	let paths = paths_from_shell_item_array(&items);
	if paths.is_empty() { None } else { Some(paths) }
}

#[allow(dead_code)]
pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let _com = com::Apartment::init().ok()?;
	let file_save_dialog: IFileSaveDialog = unsafe {
		CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER).ok()?
	};
	let file_dialog: IFileDialog = file_save_dialog.cast().ok()?;
	show_dialog(&file_dialog, p.title, p.path, p.filter, p.owner, FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_OVERWRITEPROMPT)?;
	let item = unsafe { file_save_dialog.GetResult() }.ok()?;
	path_from_shell_item(&item)
}

#[allow(dead_code)]
pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	let _com = com::Apartment::init().ok()?;
	let file_open_dialog: IFileOpenDialog = unsafe {
		CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?
	};
	let file_dialog: IFileDialog = file_open_dialog.cast().ok()?;
	show_dialog(&file_dialog, p.title, p.directory, None, p.owner, FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST)?;
	let item = unsafe { file_open_dialog.GetResult() }.ok()?;
	path_from_shell_item(&item)
}

pub fn choose_folders(p: &FolderDialog<'_>) -> Option<Vec<PathBuf>> {
	let _com = com::Apartment::init().ok()?;
	let file_open_dialog: IFileOpenDialog = unsafe {
		CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?
	};
	let file_dialog: IFileDialog = file_open_dialog.cast().ok()?;
	show_dialog(&file_dialog, p.title, p.directory, None, p.owner, FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_ALLOWMULTISELECT)?;
	let items = unsafe { file_open_dialog.GetResults() }.ok()?;
	let paths = paths_from_shell_item_array(&items);
	if paths.is_empty() { None } else { Some(paths) }
}

fn show_dialog(
	dialog: &IFileDialog,
	title: &str,
	path: Option<&Path>,
	filters: Option<&[FileFilter<'_>]>,
	owner: Option<&dyn HasWindowHandle>,
	flags: FILEOPENDIALOGOPTIONS,
) -> Option<()> {
	if !title.is_empty() {
		let title = utf16cs(title);
		unsafe { dialog.SetTitle(PCWSTR(title.as_ptr())) }.ok()?;
	}

	if let Some(filters) = filters {
		let filters = build_windows_filter(filters);
		unsafe { dialog.SetFileTypes(&filters.specs) }.ok()?;
	}

	let mut options = unsafe { dialog.GetOptions() }.ok()?;
	options |= flags;
	unsafe { dialog.SetOptions(options) }.ok()?;

	if let Some(initial_path) = utils::abspath(path) {
		if initial_path.is_dir() {
			set_default_folder(dialog, initial_path.as_ref())?;
		}
		else {
			if let Some(parent) = initial_path.parent() {
				set_default_folder(dialog, parent)?;
			}
			if (flags & FOS_PICKFOLDERS) != FOS_PICKFOLDERS {
				if let Some(file_name) = initial_path.file_name().and_then(|name| name.to_str()) {
					let name = utf16cs(file_name);
					unsafe { dialog.SetFileName(PCWSTR(name.as_ptr())) }.ok()?;
				}
			}
		}
	}

	unsafe { dialog.Show(hwnd(owner)) }.ok()
}

fn shell_item_from_path(path: &[u16]) -> Option<IShellItem> {
	unsafe { SHCreateItemFromParsingName(PCWSTR(path.as_ptr()), None).ok() }
}

fn set_default_folder(dialog: &IFileDialog, directory: &Path) -> Option<()> {
	let path = utf16cs(&directory.to_string_lossy());
	let folder = shell_item_from_path(&path)?;
	unsafe { dialog.SetDefaultFolder(&folder) }.ok()?;
	Some(())
}

fn path_from_shell_item(item: &IShellItem) -> Option<PathBuf> {
	let display_name = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) }.ok()?;
	let path = unsafe {
		let ptr = display_name.0;
		if ptr.is_null() {
			None
		}
		else {
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

fn paths_from_shell_item_array(items: &IShellItemArray) -> Vec<PathBuf> {
	let count = unsafe { items.GetCount() }.ok().unwrap_or(0);
	let mut paths = Vec::with_capacity(count as usize);
	for index in 0..count {
		let Ok(item) = (unsafe { items.GetItemAt(index) }) else {
			continue;
		};
		if let Some(path) = path_from_shell_item(&item) {
			paths.push(path);
		}
	}
	paths
}

struct DialogFilters {
	_names: Vec<Vec<u16>>,
	_specs_storage: Vec<Vec<u16>>,
	specs: Vec<COMDLG_FILTERSPEC>,
}

fn build_windows_filter(filter: &[FileFilter<'_>]) -> DialogFilters {
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

	DialogFilters {
		_names: names,
		_specs_storage: specs_storage,
		specs,
	}
}
