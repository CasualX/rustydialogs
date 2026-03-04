use std::{env, fmt, io, process};
use std::io::{IsTerminal, Write};

#[derive(Copy, Clone, Debug)]
enum TestSelector {
	AllTests,
	MessageBox,
	SaveFileDialog,
	OpenFileDialog,
	FolderDialog,
	ColorPicker,
	TextInput,
	Notification,
}

fn main() {
	print_environment();

	let args: Vec<_> = env::args().skip(1).collect();
	let args = args.iter().map(|s| s.as_str()).collect::<Vec<_>>();

	let selector = if args.len() != 1 {
		prompt_selector()
	}
	else if let Some(selector) = parse_selector(args[0]) {
		selector
	}
	else {
		prompt_selector()
	};

	match selector {
		TestSelector::AllTests => {
			test_message_box();
			test_save_file_dialog();
			test_open_file_dialog();
			test_folder_dialog();
			test_color_picker();
			test_text_input();
			test_notification();
		}
		TestSelector::MessageBox => test_message_box(),
		TestSelector::SaveFileDialog => test_save_file_dialog(),
		TestSelector::OpenFileDialog => test_open_file_dialog(),
		TestSelector::FolderDialog => test_folder_dialog(),
		TestSelector::ColorPicker => test_color_picker(),
		TestSelector::TextInput => test_text_input(),
		TestSelector::Notification => test_notification(),
	}
}

fn print_environment() {
	println!("\n{}", Color("Environment", "255;214;102"));
	println!("  {}: {}", Color("OS", "170;170;170"), env::consts::OS);
	println!("  {}: {}", Color("Arch", "170;170;170"), env::consts::ARCH);
	let backend = env::var("RUSTY_DIALOGS_BACKEND");
	let backend = match &backend { Ok(value) => value as &dyn fmt::Display, Err(_) => &"(not set)" as &dyn fmt::Display };
	println!("  {}: {}", Color("RUSTY_DIALOGS_BACKEND", "170;170;170"), backend);
	println!("  {}: {}", Color("rustc", "170;170;170"), version_command("rustc", "-V"));
	println!("  {}: {}", Color("cargo", "170;170;170"), version_command("cargo", "-V"));
}

fn version_command(cmd: &str, arg: &str) -> String {
	match process::Command::new(cmd).arg(arg).output() {
		Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
		Ok(out) => format!("(failed: exit status {})", out.status),
		Err(err) => format!("(not available: {err})"),
	}
}

fn prompt_selector() -> TestSelector {
	println!("\n{}", Color("Select tests", "255;214;102"));
	println!("  Enter = all tests");
	println!("  m = MessageBox");
	println!("  s = SaveFileDialog");
	println!("  o = OpenFileDialog");
	println!("  f = FolderDialog");
	println!("  t = TextInput");
	println!("  c = ColorPicker");
	println!("  n = Notification");

	loop {
		let value = prompt_input("Choice: ");
		match parse_selector(value.trim()) {
			Some(selector) => return selector,
			None => eprintln!("{}", Color("Invalid choice, please try again.", "255;107;107")),
		}
	}
}

fn prompt_input(prompt: &str) -> String {
	print!("{prompt}");
	_ = io::stdout().flush();
	let mut line = String::new();
	if io::stdin().read_line(&mut line).is_err() {
		return String::new();
	}
	line
}

fn parse_selector(s: &str) -> Option<TestSelector> {
	match s {
		"" | "all" | "Enter" => Some(TestSelector::AllTests),
		"m" => Some(TestSelector::MessageBox),
		"s" => Some(TestSelector::SaveFileDialog),
		"o" => Some(TestSelector::OpenFileDialog),
		"f" => Some(TestSelector::FolderDialog),
		"c" => Some(TestSelector::ColorPicker),
		"t" => Some(TestSelector::TextInput),
		"n" => Some(TestSelector::Notification),
		_ => None,
	}
}

fn step<F: Fn() -> T, T: fmt::Debug + PartialEq>(description: &str, expected: T, action: F) {
	println!("\n{} {description}", Color("Step:", "255;214;102"));
	loop {
		let result = action();
		if result == expected {
			println!("  Result: {}", Color("PASS", "123;201;111"));
			break;
		}

		println!("  Result: {} - expected {expected:?}, got {result:?}", Color("FAIL", "255;107;107"));
		if !confirm("  Test failed, retry? [Y/n]: ", true) {
			println!("  {}", Color("Marked as failed.", "255;107;107"));
			break;
		}
		println!("  {}", Color("Retrying step...", "255;214;102"));
	}
}

fn test_message_box() {
	println!("\n{}", Color("==== Testing MessageBox ====", "120;190;255"));

	let icons: &[rustydialogs::MessageIcon] = &[
		rustydialogs::MessageIcon::Info,
		rustydialogs::MessageIcon::Warning,
		rustydialogs::MessageIcon::Error,
		rustydialogs::MessageIcon::Question,
	];

	let matrix: &[(rustydialogs::MessageButtons, &[Option<rustydialogs::MessageResult>])] = &[
		(rustydialogs::MessageButtons::Ok, &[Some(rustydialogs::MessageResult::Ok), None]),
		(rustydialogs::MessageButtons::OkCancel, &[Some(rustydialogs::MessageResult::Ok), Some(rustydialogs::MessageResult::Cancel), None]),
		(rustydialogs::MessageButtons::YesNo, &[Some(rustydialogs::MessageResult::Yes), Some(rustydialogs::MessageResult::No), None]),
		(rustydialogs::MessageButtons::YesNoCancel, &[Some(rustydialogs::MessageResult::Yes), Some(rustydialogs::MessageResult::No), Some(rustydialogs::MessageResult::Cancel), None]),
	];

	for &icon in icons {
		println!("\n{} Icon: {}", Color("Testing", "120;190;255"), Color(format_args!("{:?}", icon), "255;214;102"));
		let title = format!("[tests] MessageBox - {icon:?}");
		for &(buttons, results) in matrix {
			for &result in results {
				let desc = match result {
					Some(rustydialogs::MessageResult::Ok) => "Press OK.",
					Some(rustydialogs::MessageResult::Cancel) => "Press Cancel.",
					Some(rustydialogs::MessageResult::Yes) => "Press Yes.",
					Some(rustydialogs::MessageResult::No) => "Press No.",
					None => "Dismiss the dialog.",
				};
				let message = format!("Instruction: {desc}");
				let full_desc = format!("{desc}\n  Buttons: {}\n  Icon: {}", Color(format_args!("{:?}", buttons), "255;214;102"), Color(format_args!("{:?}", icon), "255;214;102"));
				step(&full_desc,
					result,
					|| rustydialogs::MessageBox {
						title: &title,
						message: &message,
						icon,
						buttons,
						owner: None,
					}.show()
				);
			}
		}
	}
}

fn test_save_file_dialog() {
	println!("\n{}", Color("==== Testing SaveFileDialog ====", "120;190;255"));

	step("Select `readme.md` and press Save.",
		Some(env::current_dir().unwrap().join("readme.md")),
		|| rustydialogs::FileDialog {
			title: "[tests] SaveFileDialog",
			path: None,
			filter: Some(&[
				rustydialogs::FileFilter {
					desc: "Markdown Files",
					patterns: &["*.md"],
				},
				rustydialogs::FileFilter {
					desc: "Text Files",
					patterns: &["*.txt"],
				},
			]),
			owner: None,
		}.save_file()
	);

	step("Dismiss the dialog.",
		None,
		|| rustydialogs::FileDialog {
			title: "[tests] Dismiss SaveFileDialog",
			path: None,
			filter: Some(&[
				rustydialogs::FileFilter {
					desc: "Text Files",
					patterns: &["*.txt"],
				},
			]),
			owner: None,
		}.save_file()
	);
}

fn test_open_file_dialog() {
	println!("\n{}", Color("==== Testing OpenFileDialog ====", "120;190;255"));

	step("Select `Cargo.toml` and press Open.",
		Some(env::current_dir().unwrap().join("Cargo.toml")),
		|| rustydialogs::FileDialog {
			title: "[tests] OpenFileDialog",
			path: None,
			filter: Some(&[
				rustydialogs::FileFilter {
					desc: "TOML Files",
					patterns: &["*.toml"],
				},
			]),
			owner: None,
		}.pick_file()
	);
	step("Select multiple files (`Cargo.toml` and `readme.md`) and press Open.",
		Some(vec![
			env::current_dir().unwrap().join("Cargo.toml"),
			env::current_dir().unwrap().join("readme.md"),
		]),
		|| rustydialogs::FileDialog {
			title: "[tests] OpenFileDialog (multiple)",
			path: None,
			filter: None,
			owner: None,
		}.pick_files()
	);
	step("Dismiss the dialog.",
		None,
		|| rustydialogs::FileDialog {
			title: "[tests] Dismiss OpenFileDialog",
			path: None,
			filter: Some(&[
				rustydialogs::FileFilter {
					desc: "TOML Files",
					patterns: &["*.toml"],
				},
			]),
			owner: None,
		}.pick_file()
	);
}

fn test_folder_dialog() {
	println!("\n{}", Color("==== Testing FolderDialog ====", "120;190;255"));

	step("Select the `src` folder and press Open.",
		Some(env::current_dir().unwrap().join("src")),
		|| rustydialogs::FolderDialog {
			title: "[tests] FolderDialog",
			directory: None,
			owner: None,
		}.show()
	);

	step("Dismiss the dialog.",
		None,
		|| rustydialogs::FolderDialog {
			title: "[tests] Dismiss FolderDialog",
			directory: None,
			owner: None,
		}.show()
	);
}

fn test_color_picker() {
	println!("\n{}", Color("==== Testing ColorPicker ====", "120;190;255"));

	step("Select pure RED (#FF0000) and press OK.",
		Some(rustydialogs::ColorValue { red: 255, green: 0, blue: 0 }),
		|| rustydialogs::ColorPicker {
			title: "[tests] ColorPicker",
			value: rustydialogs::ColorValue { red: 255, green: 0, blue: 0 },
			owner: None,
		}.show()
	);

	step("Select specific color (#4FB3A3) (79, 179, 163) and press OK.",
		Some(rustydialogs::ColorValue { red: 79, green: 179, blue: 163 }),
		|| rustydialogs::ColorPicker {
			title: "[tests] ColorPicker",
			value: rustydialogs::ColorValue { red: 255, green: 0, blue: 0 },
			owner: None,
		}.show()
	);

	step("Dismiss the dialog.",
		None,
		|| rustydialogs::ColorPicker {
			title: "[tests] Dismiss ColorPicker",
			value: rustydialogs::ColorValue { red: 255, green: 0, blue: 0 },
			owner: None,
		}.show()
	);
}

fn test_text_input() {
	println!("\n{}", Color("==== Testing TextInput ====", "120;190;255"));

	step("Enter `Hello, Rust!` and press OK.",
		Some("Hello, Rust!".to_string()),
		|| rustydialogs::TextInput {
			title: "[tests] TextInput",
			message: "Instruction: Enter `Hello, Rust!` and press OK.",
			value: "",
			mode: rustydialogs::TextInputMode::SingleLine,
			owner: None,
		}.show()
	);

	step("Enter `Password123` and press OK.",
		Some(String::from("Password123")),
		|| rustydialogs::TextInput {
			title: "[tests] TextInput",
			message: "Instruction: Enter `Password123` and press OK.",
			value: "",
			mode: rustydialogs::TextInputMode::Password,
			owner: None,
		}.show()
	);

	step("Enter these three lines and press OK.",
		Some(String::from("Line 1\nLine 2\nLine 3")),
		|| rustydialogs::TextInput {
			title: "[tests] TextInput",
			message: "Instruction: Enter these three lines and press OK.\nLine 1\nLine 2\nLine 3",
			value: "",
			mode: rustydialogs::TextInputMode::MultiLine,
			owner: None,
		}.show()
	);

	step("Dismiss the dialog.",
		None,
		|| rustydialogs::TextInput {
			title: "[tests] Dismiss TextInput",
			message: "Instruction: Dismiss the dialog (e.g. by pressing Esc or clicking the close button).",
			value: "",
			mode: rustydialogs::TextInputMode::SingleLine,
			owner: None,
		}.show()
	);
}

fn test_notification() {
	println!("\n{}", Color("==== Testing Notification ====", "120;190;255"));

	fn notify(p: &rustydialogs::Notification<'_>) {
		println!("\n{} Confirm {} appeared.", Color("Step:", "255;214;102"), Color(format_args!("{:?}", p.icon), "255;214;102"));
		loop {
			p.show();
			if confirm("Confirm notification? [Y/n]: ", true) {
				println!("  Result: {}", Color("PASS", "123;201;111"));
				break;
			}
			println!("  Result: {}", Color("FAIL", "255;107;107"));
			if !confirm("  Test failed, retry? [Y/n]: ", true) {
				break;
			}
		}
	}

	notify(&rustydialogs::Notification {
		app_id: "rustydialogs-tests",
		title: "[INFO] Notification",
		message: "This is a test notification.\nIt should appear as a native notification on your system.",
		icon: rustydialogs::MessageIcon::Info,
		duration: rustydialogs::NotifyDuration::Short,
	});

	notify(&rustydialogs::Notification {
		app_id: "rustydialogs-tests",
		title: "[WARN] Notification",
		message: "This is a test notification.\nIt should appear as a native notification on your system.",
		icon: rustydialogs::MessageIcon::Warning,
		duration: rustydialogs::NotifyDuration::Short,
	});

	notify(&rustydialogs::Notification {
		app_id: "rustydialogs-tests",
		title: "[ERROR] Notification",
		message: "This is a test notification.\nIt should appear as a native notification on your system.",
		icon: rustydialogs::MessageIcon::Error,
		duration: rustydialogs::NotifyDuration::Short,
	});

	notify(&rustydialogs::Notification {
		app_id: "rustydialogs-tests",
		title: "[QUESTION] Notification",
		message: "This is a test notification.\nIt should appear as a native notification on your system.",
		icon: rustydialogs::MessageIcon::Question,
		duration: rustydialogs::NotifyDuration::Short,
	});
}

fn confirm(prompt: &str, default: bool) -> bool {
	loop {
		let input = prompt_input(prompt);
		let value = input.trim().to_ascii_lowercase();
		if value.is_empty() {
			return default;
		}
		if value == "y" || value == "yes" {
			return true;
		}
		if value == "n" || value == "no" {
			return false;
		}
		eprintln!("{}", Color("Please answer y/yes or n/no.", "255;107;107"));
	}
}

struct Color<'a, T> {
	value: T,
	color: &'a str,
}
#[allow(non_snake_case)]
fn Color<'a, T>(value: T, color: &'a str) -> Color<'a, T> {
	Color { value, color }
}
impl<'a, T: fmt::Display> fmt::Display for Color<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if !io::stdout().is_terminal() {
			write!(f, "{}", self.value)
		}
		else {
			write!(f, "\x1b[38;2;{}m{}\x1b[0m", self.color, self.value)
		}
	}
}
