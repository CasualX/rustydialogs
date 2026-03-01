const APP_ID: &str = "rustydialogs.example.notify";

fn main() {
	// winrt-toast: Takes almost three seconds to show the first notification due to registration delays.
	// All Notifications shown before the registration is complete will be ignored.
	rustydialogs::Notification::setup(APP_ID);

	let i = std::process::id() / 4;

	let icon = match i % 4 {
		0 => rustydialogs::MessageIcon::Info,
		1 => rustydialogs::MessageIcon::Warning,
		2 => rustydialogs::MessageIcon::Error,
		_ => rustydialogs::MessageIcon::Question,
	};

	let notify = rustydialogs::Notification {
		app_id: APP_ID,
		title: "Rusty Dialogs",
		message: "This is a native notification.",
		icon,
		timeout: rustydialogs::Notification::SHORT_TIMEOUT,
	};

	notify.show();
	println!("Notification shown");

	// winrt-toast: Wait a bit to ensure the notification is visible before the program exits.
	std::thread::sleep(std::time::Duration::from_millis(100));
}
