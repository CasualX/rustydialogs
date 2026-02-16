fn main() {
	let popup = rustydialogs::NotifyPopup {
		title: "Rusty Dialogs",
		message: "This is a native notification popup.",
		icon: rustydialogs::MessageIcon::Info,
		timeout: 5000,
	};

	popup.show();
	println!("NotifyPopup shown");
}
