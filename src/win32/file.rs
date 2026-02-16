use windows::core::{PCWSTR, PWSTR};
use windows::Win32::UI::Controls::Dialogs::{
	GetOpenFileNameW, GetSaveFileNameW, OPENFILENAMEW, OFN_ALLOWMULTISELECT, OFN_EXPLORER,
	OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST,
};

use super::*;
use std::path::{Path, PathBuf};

pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	pick_files_impl(p, false)
		.and_then(|mut paths| if paths.is_empty() { None } else { Some(paths.remove(0)) })
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	pick_files_impl(p, true)
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let title = utf16_null_terminated(p.title);
	let filter = build_windows_filter(p.filter);
	let default_path_and_file = default_path_and_file(p);
	let mut file_buffer = initial_file_buffer(default_path_and_file.as_deref());

	let mut open_file_name = OPENFILENAMEW::default();
	open_file_name.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
	open_file_name.lpstrTitle = PCWSTR(title.as_ptr());
	if let Some(filter) = &filter {
		open_file_name.lpstrFilter = PCWSTR(filter.as_ptr());
	}
	open_file_name.lpstrFile = PWSTR(file_buffer.as_mut_ptr());
	open_file_name.nMaxFile = file_buffer.len() as u32;
	open_file_name.Flags = OFN_EXPLORER | OFN_NOCHANGEDIR | OFN_PATHMUSTEXIST | OFN_OVERWRITEPROMPT;

	let selected = unsafe { GetSaveFileNameW(&mut open_file_name).as_bool() };
	if !selected {
		return None;
	}

	wide_to_string_until_nul(&file_buffer)
}

fn pick_files_impl(p: &FileDialog<'_>, allow_multiple_selects: bool) -> Option<Vec<PathBuf>> {
	let title = utf16_null_terminated(p.title);
	let filter = build_windows_filter(p.filter);
	let default_path_and_file = default_path_and_file(p);
	let mut file_buffer = initial_file_buffer(default_path_and_file.as_deref());

	let mut open_file_name = OPENFILENAMEW::default();
	open_file_name.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
	open_file_name.lpstrTitle = PCWSTR(title.as_ptr());
	if let Some(filter) = &filter {
		open_file_name.lpstrFilter = PCWSTR(filter.as_ptr());
	}
	open_file_name.lpstrFile = PWSTR(file_buffer.as_mut_ptr());
	open_file_name.nMaxFile = file_buffer.len() as u32;
	open_file_name.Flags = OFN_EXPLORER
		| OFN_NOCHANGEDIR
		| OFN_PATHMUSTEXIST
		| OFN_FILEMUSTEXIST
		| if allow_multiple_selects {
			OFN_ALLOWMULTISELECT
		} else {
			Default::default()
		};

	let selected = unsafe { GetOpenFileNameW(&mut open_file_name).as_bool() };
	if !selected {
		return None;
	}

	let paths = parse_open_file_buffer(&file_buffer);
	if paths.is_empty() {
		None
	} else {
		Some(paths)
	}
}

fn initial_file_buffer(default_path_and_file: Option<&Path>) -> Vec<u16> {
	let mut buffer = vec![0u16; 16 * 1024];
	let Some(default_path_and_file) = default_path_and_file else {
		return buffer;
	};

	if default_path_and_file.as_os_str().is_empty() {
		return buffer;
	}

	let encoded = utf16_null_terminated(&default_path_and_file.to_string_lossy());
	let copy_len = encoded.len().min(buffer.len());
	buffer[..copy_len].copy_from_slice(&encoded[..copy_len]);
	buffer
}

fn default_path_and_file(dialog: &FileDialog<'_>) -> Option<PathBuf> {
	match (dialog.directory, dialog.file_name) {
		(Some(dir), Some(file)) if !dir.as_os_str().is_empty() && !file.as_os_str().is_empty() => {
			Some(dir.join(file))
		}
		(Some(dir), _) if !dir.as_os_str().is_empty() => Some(dir.to_path_buf()),
		(_, Some(file)) if !file.as_os_str().is_empty() => Some(file.to_path_buf()),
		_ => None,
	}
}

fn build_windows_filter(filter: Option<&[FileFilter<'_>]>) -> Option<Vec<u16>> {
	let filter = filter?;
	if filter.is_empty() {
		return None;
	}

	let mut spec = String::new();
	for entry in filter {
		_ = write!(spec, "{}\0{}\0", entry.desc, utils::PrintJoin { parts: entry.patterns, separator: ";" });
	}
	spec.push_str("All Files\0*.*\0\0");
	Some(spec.encode_utf16().collect())
}

fn wide_to_string_until_nul(input: &[u16]) -> Option<PathBuf> {
	let length = input.iter().position(|value| *value == 0)?;
	if length == 0 {
		return None;
	}
	Some(PathBuf::from(String::from_utf16_lossy(&input[..length])))
}

fn parse_open_file_buffer(input: &[u16]) -> Vec<PathBuf> {
	let mut segments = Vec::new();
	let mut start = 0usize;

	for index in 0..input.len() {
		if input[index] != 0 {
			continue;
		}

		if index == start {
			break;
		}

		segments.push(PathBuf::from(String::from_utf16_lossy(&input[start..index])));
		start = index + 1;
	}

	if segments.is_empty() {
		return Vec::new();
	}

	if segments.len() == 1 {
		return vec![segments.remove(0)];
	}

	let directory = segments.remove(0);
	segments
		.iter()
		.map(|file_name| directory.join(file_name))
		.collect()
}
