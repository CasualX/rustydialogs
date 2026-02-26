const APP_ID: &str = "rustydialogs.example.notify";

fn main() {
	rustydialogs::Notification::setup(APP_ID);

	// winrt-toast: Takes almost three seconds to show the first notification due to registration delays.
	// All Notifications shown before the first one may be ignored...

	let notify = rustydialogs::Notification {
		app_id: APP_ID,
		title: "Rusty Dialogs",
		message: "This is a native notification.",
		icon: rustydialogs::MessageIcon::Info,
		timeout: rustydialogs::Notification::SHORT_TIMEOUT,
	};

	notify.show();
	println!("Notification shown");

	// winrt-toast: Wait a bit to ensure the notification is visible before the program exits.
	std::thread::sleep(std::time::Duration::from_millis(100));
}
