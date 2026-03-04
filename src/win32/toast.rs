use std::{env, mem, ptr};
use std::sync::OnceLock;
use windows::core::{w, Error, GUID, HSTRING, Interface, PCWSTR, PWSTR};
use windows::Data::Xml::Dom::XmlDocument;
use windows::Win32::Foundation::{PROPERTYKEY, RPC_E_CHANGED_MODE};
use windows::Win32::System::Com::{
	CoCreateInstance, CoInitializeEx, CoTaskMemAlloc, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
	COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PROPVARIANT};
use windows::Win32::System::Variant::VT_LPWSTR;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

use super::*;

pub fn setup(app_id: &str) -> bool {
	static SETUP_ONCE: OnceLock<bool> = OnceLock::new();

	*SETUP_ONCE.get_or_init(|| {
		if let Err(error) = ensure_start_menu_shortcut(app_id) {
			eprintln!("rustydialogs: failed to register toast shortcut during setup: {error}");
			return false;
		}

		true
	})
}

pub fn notify(p: &Notification<'_>) {
	if !setup(p.app_id) {
		return;
	}

	if let Err(error) = try_notify(p) {
		eprintln!("rustydialogs: WinRT toast notify failed: {error}");
	}
}

fn try_notify(p: &Notification<'_>) -> windows::core::Result<()> {
	let xml = toast_xml(p);

	let document = XmlDocument::new()?;
	document.LoadXml(&HSTRING::from(xml))?;

	let toast = ToastNotification::CreateToastNotification(&document)?;
	let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(p.app_id))?;
	notifier.Show(&toast)?;

	Ok(())
}

fn ensure_start_menu_shortcut(app_id: &str) -> windows::core::Result<()> {
	if app_id.is_empty() {
		return Err(Error::new(windows::core::HRESULT(0x80070057u32 as i32), "Application identifier cannot be empty"));
	}

	let Ok(exe_path) = env::current_exe() else {
		return Err(Error::new(windows::core::HRESULT(0x80070002u32 as i32), "Failed to get current executable path"));
	};
	let Some(programs_dir) = start_menu_programs_dir() else {
		return Err(Error::new(windows::core::HRESULT(0x80070002u32 as i32), "Failed to get Start Menu Programs directory"));
	};
	// let _ = fs::create_dir_all(&programs_dir);

	let shortcut_name = sanitize_shortcut_name(app_id);
	let shortcut_path = programs_dir.join(format!("{shortcut_name}.lnk"));

	// If the shortcut already exists, recreate it to ensure the AppUserModelID is up to date (required for toasts to work).

	let _com = ComApartment::init()?;

	let shell_link: IShellLinkW = unsafe {
		CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?
	};

	let exe_utf16 = utf16cs(&exe_path.to_string_lossy());
	unsafe { shell_link.SetPath(PCWSTR(exe_utf16.as_ptr()))?; }
	unsafe { shell_link.SetArguments(w!(""))?; }

	if let Some(workdir) = exe_path.parent() {
		let workdir_utf16 = utf16cs(&workdir.to_string_lossy());
		unsafe { shell_link.SetWorkingDirectory(PCWSTR(workdir_utf16.as_ptr()))?; }
	}

	let desc_utf16 = utf16cs(app_id);
	unsafe { shell_link.SetDescription(PCWSTR(desc_utf16.as_ptr()))?; }

	let property_store: IPropertyStore = shell_link.cast()?;
	set_app_user_model_id(&property_store, app_id)?;

	let persist: IPersistFile = shell_link.cast()?;
	let shortcut_utf16 = utf16cs(&shortcut_path.to_string_lossy());
	unsafe { persist.Save(PCWSTR(shortcut_utf16.as_ptr()), true)?; }

	Ok(())
}

fn set_app_user_model_id(property_store: &IPropertyStore, app_id: &str) -> windows::core::Result<()> {
	let app_id_wide = utf16cs(app_id);
	let byte_len = app_id_wide.len() * mem::size_of::<u16>();
	let memory = unsafe { CoTaskMemAlloc(byte_len) } as *mut u16;
	if memory.is_null() {
		return Err(Error::new(windows::core::HRESULT(0x8007000Eu32 as i32), "CoTaskMemAlloc failed"));
	}

	unsafe {
		ptr::copy_nonoverlapping(app_id_wide.as_ptr(), memory, app_id_wide.len());
	}

	let mut value = PROPVARIANT::default();
	unsafe {
		let var = &mut value.Anonymous.Anonymous;
		var.vt = VT_LPWSTR;
		var.Anonymous.pwszVal = PWSTR(memory);
	}

	let set_result = unsafe {
		property_store.SetValue(&PKEY_APP_USER_MODEL_ID, &value)
			.and_then(|_| property_store.Commit())
	};

	unsafe {
		let _ = PropVariantClear(&mut value);
	}

	set_result
}

fn start_menu_programs_dir() -> Option<PathBuf> {
	let mut appdata = PathBuf::from(env::var_os("APPDATA")?);
	appdata.push("Microsoft");
	appdata.push("Windows");
	appdata.push("Start Menu");
	appdata.push("Programs");
	Some(appdata)
}

fn sanitize_shortcut_name(app_id: &str) -> String {
	app_id.replace(|ch| matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'), "_")
}

const PKEY_APP_USER_MODEL_ID: PROPERTYKEY = PROPERTYKEY {
	fmtid: GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3),
	pid: 5,
};

struct ComApartment {
	should_uninitialize: bool,
}

impl ComApartment {
	fn init() -> windows::core::Result<Self> {
		let result = unsafe {
			CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
		};

		if result.is_ok() {
			Ok(Self {
				should_uninitialize: true,
			})
		}
		else if result == RPC_E_CHANGED_MODE {
			Ok(Self {
				should_uninitialize: false,
			})
		}
		else {
			Err(windows::core::Error::from(result))
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

fn toast_xml(p: &Notification<'_>) -> String {
	let duration = if p.timeout > 0 && p.timeout <= 7000 { "short" } else { "long" };
	let title = xml_escape(p.title);
	let message = xml_escape(p.message);
	let icon = match p.icon {
		MessageIcon::Info => "ℹ",
		MessageIcon::Warning => "⚠",
		MessageIcon::Error => "❌",
		MessageIcon::Question => "❔",
	};

	format!(
		r#"<toast duration="{duration}">
	<visual>
		<binding template="ToastGeneric">
			<text>{icon} {title}</text>
			<text>{message}</text>
		</binding>
	</visual>
</toast>"#
	)
}

fn xml_escape(value: &str) -> String {
	let mut result = String::with_capacity(value.len());
	for ch in value.chars() {
		match ch {
			'&' => result.push_str("&amp;"),
			'<' => result.push_str("&lt;"),
			'>' => result.push_str("&gt;"),
			'\"' => result.push_str("&quot;"),
			'\'' => result.push_str("&apos;"),
			_ => result.push(ch),
		}
	}
	result
}
