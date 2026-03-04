use std::sync::OnceLock;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{
	Shell_NotifyIconW, ExtractIconExW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_ERROR, NIIF_INFO,
	NIIF_WARNING, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
	NOTIFY_ICON_INFOTIP_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::{
	CreateWindowExW, DestroyIcon, DestroyWindow, HICON, IDI_APPLICATION, LoadIconW,
	WINDOW_EX_STYLE, WINDOW_STYLE,
};
use windows::core::{w, Error, PCWSTR};

use super::*;

const CLASS_NAME: PCWSTR = w!("Static");
const TRAY_ICON_ID: u32 = 1;

struct TrayState {
	hwnd: HWND,
	icon: Icon,
}

unsafe impl Send for TrayState {}
unsafe impl Sync for TrayState {}

fn init(app_id: &str) -> Option<&TrayState> {
	static TRAY_STATE: OnceLock<Option<TrayState>> = OnceLock::new();

	TRAY_STATE.get_or_init(|| {
		match TrayState::create(app_id) {
			Ok(state) => Some(state),
			Err(error) => {
				eprintln!("rustydialogs: failed to initialize tray notifications: {error}");
				None
			},
		}
	}).as_ref()
}

pub fn setup(app_id: &str) -> bool {
	init(app_id).is_some()
}

pub fn notify(p: &Notification<'_>) {
	let Some(state) = init(p.app_id) else {
		return;
	};

	let mut data = state.base_data();
	data.uFlags = NIF_INFO;
	copy_wide_trunc(&mut data.szInfoTitle, p.title);
	copy_wide_trunc(&mut data.szInfo, p.message);
	data.dwInfoFlags = icon_to_flags(p.icon);
	data.Anonymous.uTimeout = timeout_hint(p.duration);

	unsafe {
		let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
	}
}

impl TrayState {
	fn create(app_id: &str) -> windows::core::Result<Self> {
		if app_id.is_empty() {
			return Err(Error::new(windows::core::HRESULT(0x80070057u32 as i32), "Application identifier cannot be empty"));
		}

		let hwnd = unsafe {
			CreateWindowExW(
				WINDOW_EX_STYLE::default(),
				CLASS_NAME, w!(""),
				WINDOW_STYLE::default(),
				0, 0, 0, 0,
				None, None, None, None,
			)
		}?;

		let icon = Icon::load();

		let mut data = NOTIFYICONDATAW::default();
		data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
		data.hWnd = hwnd;
		data.uID = TRAY_ICON_ID;
		data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
		data.uCallbackMessage = 0;
		data.hIcon = icon.hicon;
		copy_wide_trunc(&mut data.szTip, app_id);

		unsafe {
			if !Shell_NotifyIconW(NIM_ADD, &data).as_bool() {
				let error = windows::core::Error::from_thread();
				let _ = DestroyWindow(hwnd);
				return Err(error);
			}
		}

		Ok(Self { hwnd, icon })
	}

	fn base_data(&self) -> NOTIFYICONDATAW {
		let mut data = NOTIFYICONDATAW::default();
		data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
		data.hWnd = self.hwnd;
		data.uID = TRAY_ICON_ID;
		data.hIcon = self.icon.hicon;
		data
	}
}

impl Drop for TrayState {
	fn drop(&mut self) {
		let data = self.base_data();
		unsafe {
			let _ = Shell_NotifyIconW(NIM_DELETE, &data);
			if !self.hwnd.is_invalid() {
				let _ = DestroyWindow(self.hwnd);
			}
		}
	}
}

fn timeout_hint(duration: NotifyDuration) -> u32 {
	match duration {
		NotifyDuration::Short => 5000,
		NotifyDuration::Long => 10000,
		NotifyDuration::Infinite => 0,
	}
}

fn icon_to_flags(icon: MessageIcon) -> NOTIFY_ICON_INFOTIP_FLAGS {
	match icon {
		MessageIcon::Info => NIIF_INFO,
		MessageIcon::Warning => NIIF_WARNING,
		MessageIcon::Error => NIIF_ERROR,
		MessageIcon::Question => NIIF_INFO,
	}
}

fn copy_wide_trunc(dest: &mut [u16], value: &str) {
	if dest.is_empty() {
		return;
	}

	let mut i = 0usize;
	for unit in value.encode_utf16() {
		if i + 1 >= dest.len() {
			break;
		}
		dest[i] = unit;
		i += 1;
	}
	dest[i] = 0;
}

struct Icon {
	hicon: HICON,
	owned: bool,
}
impl Default for Icon {
	fn default() -> Icon {
		let hicon = unsafe { LoadIconW(None, IDI_APPLICATION).unwrap_or_default() };
		Icon { hicon, owned: false }
	}
}
impl Icon {
	fn load() -> Icon {
		load_exe_icon()
			.map(|icon| Icon { hicon: icon, owned: true })
			.unwrap_or_default()
	}
}
impl Drop for Icon {
	fn drop(&mut self) {
		if self.owned && self.hicon != HICON::default() {
			let _ = unsafe { DestroyIcon(self.hicon) };
		}
	}
}

fn load_exe_icon() -> Option<HICON> {
	let exe = std::env::current_exe().ok()?;
	let exe_wide = utf16cs(&exe.to_string_lossy());
	let mut large = [HICON::default(); 1];
	let mut small = [HICON::default(); 1];

	let count = unsafe {
		ExtractIconExW(PCWSTR(exe_wide.as_ptr()), 0, Some(large.as_mut_ptr()), Some(small.as_mut_ptr()), 1)
	};

	if count == 0 {
		return None;
	}

	let [small_icon] = small;
	let [large_icon] = large;

	if small_icon != HICON::default() {
		if large_icon != HICON::default() {
			unsafe {
				let _ = DestroyIcon(large_icon);
			}
		}
		Some(small_icon)
	}
	else if large_icon != HICON::default() {
		Some(large_icon)
	}
	else {
		None
	}
}
