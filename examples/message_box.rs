fn main() {
	let i = std::process::id() / 4;

	let icon = match i % 4 {
		0 => rustydialogs::MessageIcon::Info,
		1 => rustydialogs::MessageIcon::Warning,
		2 => rustydialogs::MessageIcon::Error,
		_ => rustydialogs::MessageIcon::Question,
	};

	let buttons = match i / 4 % 4 {
		0 => rustydialogs::MessageButtons::Ok,
		1 => rustydialogs::MessageButtons::OkCancel,
		2 => rustydialogs::MessageButtons::YesNo,
		_ => rustydialogs::MessageButtons::YesNoCancel,
	};

	let dialog = rustydialogs::MessageBox {
		title: "Rusty Dialogs",
		message: "Hello from Rust!\nThis is a native message box.",
		icon,
		buttons,
		owner: None,
	};

	let selected = dialog.show();
	println!("Selected: {selected:?}");
}
