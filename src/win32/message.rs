use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{
	MessageBoxW, IDCANCEL, IDNO, IDOK, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION,
	MB_ICONWARNING, MB_OK, MB_OKCANCEL, MB_YESNO, MB_YESNOCANCEL,
};

use super::*;

pub fn show(p: &MessageBox<'_>) -> Option<MessageResult> {
		let title = utf16_null_terminated(p.title);
		let message = utf16_null_terminated(p.message);
		let icon = match p.icon {
			MessageIcon::Info => MB_ICONINFORMATION,
			MessageIcon::Warning => MB_ICONWARNING,
			MessageIcon::Error => MB_ICONERROR,
			MessageIcon::Question => MB_ICONQUESTION,
		};
		let buttons = match p.buttons {
			MessageButtons::Ok => MB_OK,
			MessageButtons::OkCancel => MB_OKCANCEL,
			MessageButtons::YesNo => MB_YESNO,
			MessageButtons::YesNoCancel => MB_YESNOCANCEL,
		};

		let result = unsafe {
			MessageBoxW(
				None,
				PCWSTR(message.as_ptr()),
				PCWSTR(title.as_ptr()),
				buttons | icon,
			)
		};

		match result {
			IDOK => Some(MessageResult::Ok),
			IDCANCEL => Some(MessageResult::Cancel),
			IDYES => Some(MessageResult::Yes),
			IDNO => Some(MessageResult::No),
			_ => Some(default_selection(p.buttons)),
		}
}

fn default_selection(buttons: MessageButtons) -> MessageResult {
	match buttons {
		MessageButtons::Ok => MessageResult::Ok,
		MessageButtons::OkCancel => MessageResult::Cancel,
		MessageButtons::YesNo => MessageResult::No,
		MessageButtons::YesNoCancel => MessageResult::Cancel,
	}
}
