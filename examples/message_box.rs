fn main() {
	let dialog = rustydialogs::MessageBox {
		title: "Rusty Dialogs",
		message: "Hello from Rust!\nThis is a native message box.",
		icon: rustydialogs::MessageIcon::Info,
		buttons: rustydialogs::MessageButtons::Ok,
	};

	let selected = dialog.show();
	println!("Selected: {selected:?}");
}
