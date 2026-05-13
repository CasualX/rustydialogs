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

struct Config {
	selector: TestSelector,
	retry_on_fail: bool,
}

struct Runner {
	retry_on_fail: bool,
	failures: usize,
}

fn main() {
	print_environment();

	let config = parse_config(env::args().skip(1).collect());
	let mut runner = Runner {
		retry_on_fail: config.retry_on_fail,
		failures: 0,
	};

	match config.selector {
		TestSelector::AllTests => {
			runner.test_message_box();
			runner.test_save_file_dialog();
			runner.test_open_file_dialog();
			runner.test_folder_dialog();
			runner.test_color_picker();
			runner.test_text_input();
			runner.test_notification();
		}
		TestSelector::MessageBox => runner.test_message_box(),
		TestSelector::SaveFileDialog => runner.test_save_file_dialog(),
		TestSelector::OpenFileDialog => runner.test_open_file_dialog(),
		TestSelector::FolderDialog => runner.test_folder_dialog(),
		TestSelector::ColorPicker => runner.test_color_picker(),
		TestSelector::TextInput => runner.test_text_input(),
		TestSelector::Notification => runner.test_notification(),
	}

	if runner.failures == 0 {
		println!("\n{}", Color("All requested steps passed.", "123;201;111"));
	}
	else {
		eprintln!("\n{} {}", Color("Failures:", "255;107;107"), runner.failures);
		process::exit(1);
	}
}

fn parse_config(args: Vec<String>) -> Config {
	let mut retry_on_fail = true;
	let mut selector = None;

	for arg in args {
		match arg.as_str() {
			"--no-retry" => retry_on_fail = false,
			"--retry" => retry_on_fail = true,
			_ => match parse_selector(&arg) {
				Some(value) => selector = Some(value),
				None => {
					eprintln!("{} {arg}", Color("Ignoring unknown argument:", "255;107;107"));
				}
			},
		}
	}

	Config {
		selector: selector.unwrap_or_else(prompt_selector),
		retry_on_fail,
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

impl Runner {
	fn step<F: Fn() -> T, T: fmt::Debug + PartialEq>(&mut self, description: &str, expected: T, action: F) {
		println!("\n{} {description}", Color("Step:", "255;214;102"));
		loop {
			let result = action();
			if result == expected {
				println!("  Result: {}", Color("PASS", "123;201;111"));
				break;
			}

			println!("  Result: {} - expected {expected:?}, got {result:?}", Color("FAIL", "255;107;107"));
			if !self.retry_on_fail || !confirm("  Test failed, retry? [Y/n]: ", true) {
				println!("  {}", Color("Marked as failed.", "255;107;107"));
				self.failures += 1;
				break;
			}
			println!("  {}", Color("Retrying step...", "255;214;102"));
		}
	}
}

impl Runner {
	fn test_message_box(&mut self) {
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
					self.step(&full_desc,
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
}

fn sorted<T: Ord>(mut items: Vec<T>) -> Vec<T> { items.sort(); items }

impl Runner {
	fn test_save_file_dialog(&mut self) {
		println!("\n{}", Color("==== Testing SaveFileDialog ====", "120;190;255"));

		let current_dir = env::current_dir().unwrap();

		self.step("Select `readme.md` and press Save.",
			Some(current_dir.join("readme.md")),
			|| rustydialogs::FileDialog {
				title: "[tests] SaveFileDialog",
				path: Some(&current_dir),
				filters: Some(&[
					rustydialogs::FileFilter {
						name: "Markdown Files",
						patterns: &["*.md"],
					},
					rustydialogs::FileFilter {
						name: "Text Files",
						patterns: &["*.txt"],
					},
				]),
				owner: None,
			}.save_file()
		);

		self.step("Dismiss the dialog.",
			None,
			|| rustydialogs::FileDialog {
				title: "[tests] Dismiss SaveFileDialog",
				path: Some(&current_dir),
				filters: Some(&[
					rustydialogs::FileFilter {
						name: "Text Files",
						patterns: &["*.txt"],
					},
				]),
				owner: None,
			}.save_file()
		);
	}
}

impl Runner {
	fn test_open_file_dialog(&mut self) {
		println!("\n{}", Color("==== Testing OpenFileDialog ====", "120;190;255"));

		let current_dir = env::current_dir().unwrap();

		self.step("Select `Cargo.toml` and press Open.",
			Some(current_dir.join("Cargo.toml")),
			|| rustydialogs::FileDialog {
				title: "[tests] OpenFileDialog",
				path: Some(&current_dir),
				filters: Some(&[
					rustydialogs::FileFilter {
						name: "TOML Files",
						patterns: &["*.toml"],
					},
				]),
				owner: None,
			}.pick_file()
		);

		self.step("Select multiple files (`Cargo.toml` and `readme.md`) and press Open.",
			Some(vec![
				current_dir.join("Cargo.toml"),
				current_dir.join("readme.md"),
			]),
			|| rustydialogs::FileDialog {
				title: "[tests] OpenFileDialog (multiple)",
				path: Some(&current_dir),
				filters: None,
				owner: None,
			}.pick_files().map(sorted)
		);

		self.step("Dismiss the dialog.",
			None,
			|| rustydialogs::FileDialog {
				title: "[tests] Dismiss OpenFileDialog",
				path: Some(&current_dir),
				filters: Some(&[
					rustydialogs::FileFilter {
						name: "TOML Files",
						patterns: &["*.toml"],
					},
				]),
				owner: None,
			}.pick_file()
		);
	}
}

impl Runner {
	fn test_folder_dialog(&mut self) {
		println!("\n{}", Color("==== Testing FolderDialog ====", "120;190;255"));

		let current_dir = env::current_dir().unwrap();

		self.step("Select the `src` folder and press Open.",
			Some(current_dir.join("src")),
			|| rustydialogs::FileDialog {
				title: "[tests] FileDialog choose_folder",
				path: Some(&current_dir),
				filters: None,
				owner: None,
			}.choose_folder()
		);

		self.step("Select multiple folders (`src` and `examples`) and press Open.",
			Some(vec![
				current_dir.join("examples"),
				current_dir.join("src"),
			]),
			|| rustydialogs::FileDialog {
				title: "[tests] FileDialog choose_folders",
				path: Some(&current_dir),
				filters: None,
				owner: None,
			}.choose_folders().map(sorted)
		);

		self.step("Dismiss the dialog.",
			None,
			|| rustydialogs::FileDialog {
				title: "[tests] Dismiss FileDialog choose_folder",
				path: Some(&current_dir),
				filters: None,
				owner: None,
			}.choose_folder()
		);
	}
}

impl Runner {
	fn test_color_picker(&mut self) {
		println!("\n{}", Color("==== Testing ColorPicker ====", "120;190;255"));

		self.step("Select pure RED (#FF0000) and press OK.",
			Some(rustydialogs::ColorValue { red: 255, green: 0, blue: 0 }),
			|| rustydialogs::ColorPicker {
				title: "[tests] ColorPicker",
				value: rustydialogs::ColorValue { red: 255, green: 0, blue: 0 },
				owner: None,
			}.show()
		);

		self.step("Select specific color (#4FB3A3) (79, 179, 163) and press OK.",
			Some(rustydialogs::ColorValue { red: 79, green: 179, blue: 163 }),
			|| rustydialogs::ColorPicker {
				title: "[tests] ColorPicker",
				value: rustydialogs::ColorValue { red: 255, green: 0, blue: 0 },
				owner: None,
			}.show()
		);

		self.step("Dismiss the dialog.",
			None,
			|| rustydialogs::ColorPicker {
				title: "[tests] Dismiss ColorPicker",
				value: rustydialogs::ColorValue { red: 255, green: 0, blue: 0 },
				owner: None,
			}.show()
		);
	}
}

impl Runner {
	fn test_text_input(&mut self) {
		println!("\n{}", Color("==== Testing TextInput ====", "120;190;255"));

		self.step("Enter `Hello, Rust!` and press OK.",
			Some("Hello, Rust!".to_string()),
			|| rustydialogs::TextInput {
				title: "[tests] TextInput",
				message: "Instruction: Enter `Hello, Rust!` and press OK.",
				value: "",
				mode: rustydialogs::TextInputMode::SingleLine,
				owner: None,
			}.show()
		);

		self.step("Enter `Password123` and press OK.",
			Some(String::from("Password123")),
			|| rustydialogs::TextInput {
				title: "[tests] TextInput",
				message: "Instruction: Enter `Password123` and press OK.",
				value: "",
				mode: rustydialogs::TextInputMode::Password,
				owner: None,
			}.show()
		);

		self.step("Enter these three lines and press OK.",
			Some(String::from("Line 1\nLine 2\nLine 3")),
			|| rustydialogs::TextInput {
				title: "[tests] TextInput",
				message: "Instruction: Enter these three lines and press OK.\nLine 1\nLine 2\nLine 3",
				value: "",
				mode: rustydialogs::TextInputMode::MultiLine,
				owner: None,
			}.show()
		);

		self.step("Dismiss the dialog.",
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
}

impl Runner {
	fn notify(&mut self, p: &rustydialogs::Notification<'_>) {
		println!("\n{} Confirm {} appeared.", Color("Step:", "255;214;102"), Color(format_args!("{:?}", p.icon), "255;214;102"));
		loop {
			p.show();
			if confirm("Confirm notification? [Y/n]: ", true) {
				println!("  Result: {}", Color("PASS", "123;201;111"));
				break;
			}
			println!("  Result: {}", Color("FAIL", "255;107;107"));
			if !self.retry_on_fail || !confirm("  Test failed, retry? [Y/n]: ", true) {
				self.failures += 1;
				break;
			}
		}
	}

	fn test_notification(&mut self) {
		println!("\n{}", Color("==== Testing Notification ====", "120;190;255"));


		self.notify(&rustydialogs::Notification {
			app_id: "rustydialogs-tests",
			title: "[INFO] Notification",
			message: "This is a test notification.\nIt should appear as a native notification on your system.",
			icon: rustydialogs::MessageIcon::Info,
			duration: rustydialogs::NotifyDuration::Short,
		});

		self.notify(&rustydialogs::Notification {
			app_id: "rustydialogs-tests",
			title: "[WARN] Notification",
			message: "This is a test notification.\nIt should appear as a native notification on your system.",
			icon: rustydialogs::MessageIcon::Warning,
			duration: rustydialogs::NotifyDuration::Short,
		});

		self.notify(&rustydialogs::Notification {
			app_id: "rustydialogs-tests",
			title: "[ERROR] Notification",
			message: "This is a test notification.\nIt should appear as a native notification on your system.",
			icon: rustydialogs::MessageIcon::Error,
			duration: rustydialogs::NotifyDuration::Short,
		});

		self.notify(&rustydialogs::Notification {
			app_id: "rustydialogs-tests",
			title: "[QUESTION] Notification",
			message: "This is a test notification.\nIt should appear as a native notification on your system.",
			icon: rustydialogs::MessageIcon::Question,
			duration: rustydialogs::NotifyDuration::Short,
		});
	}
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
