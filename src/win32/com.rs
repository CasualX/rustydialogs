use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{
	CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};

pub struct Apartment {
	should_uninitialize: bool,
}

impl Apartment {
	pub fn init() -> windows::core::Result<Self> {
		let result = unsafe {
			CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
		};

		if result.is_ok() {
			Ok(Self {
				should_uninitialize: true,
			})
		} else if result == RPC_E_CHANGED_MODE {
			Ok(Self {
				should_uninitialize: false,
			})
		} else {
			Err(windows::core::Error::from(result))
		}
	}
}

impl Drop for Apartment {
	fn drop(&mut self) {
		if self.should_uninitialize {
			unsafe {
				CoUninitialize();
			}
		}
	}
}
