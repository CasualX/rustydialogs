use std::env;
use std::error::Error;
use std::path::PathBuf;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::{Window, WindowId};

const APP_ID: &str = "rustydialogs.example.winit";
const BASE_TITLE: &str = "Rusty Dialogs + winit";

fn main() -> Result<(), Box<dyn Error>> {
	let event_loop = EventLoop::new()?;
	let mut app = App::new(env::current_dir()?);
	Ok(event_loop.run_app(&mut app)?)
}

#[derive(Debug)]
struct App {
	current_dir: PathBuf,
	window: Option<Window>,
}

impl App {
	fn new(current_dir: PathBuf) -> Self {
		Self {
			current_dir,
			window: None,
		}
	}

	fn owner(&self) -> Option<&dyn raw_window_handle::HasWindowHandle> {
		self.window.as_ref().map(|window| window as &dyn raw_window_handle::HasWindowHandle)
	}

	fn set_status(&self, status: &str) {
		println!("{status}");
		if let Some(window) = self.window.as_ref() {
			window.set_title(&format!("{BASE_TITLE} - {status}"));
		}
	}

	fn print_help(&self) {
		println!("\n{BASE_TITLE}");
		println!("Focus the window, then press a key to open a native dialog owned by the winit window:");
		println!("  M = MessageBox");
		println!("  O = OpenFileDialog (single file)");
		println!("  A = OpenFileDialog (multiple files)");
		println!("  S = SaveFileDialog");
		println!("  F = FolderDialog (single folder)");
		println!("  G = FolderDialog (multiple folders)");
		println!("  C = ColorPicker");
		println!("  T = TextInput (single line)");
		println!("  P = TextInput (password)");
		println!("  L = TextInput (multi-line)");
		println!("  N = Notification");
		println!("  H = Print this help");
		println!("  Q or Esc = Quit");
		self.set_status("ready");
	}

	fn show_message_box(&self) {
		let result = rustydialogs::MessageBox {
			title: "Rusty Dialogs + winit",
			message: "This dialog is owned by the active winit window.",
			icon: rustydialogs::MessageIcon::Question,
			buttons: rustydialogs::MessageButtons::YesNoCancel,
			owner: self.owner(),
		}.show();
		self.set_status(&format!("message box -> {result:?}"));
	}

	fn open_file(&self) {
		let filters = [
			rustydialogs::FileFilter {
				name: "Rust Sources",
				patterns: &["*.rs"],
			},
			rustydialogs::FileFilter {
				name: "TOML Files",
				patterns: &["*.toml"],
			},
		];
		let result = rustydialogs::FileDialog {
			title: "Open a file from the winit example",
			path: Some(&self.current_dir),
			filters: Some(&filters),
			owner: self.owner(),
		}.pick_file();
		self.set_status(&format!("open file -> {result:?}"));
	}

	fn open_files(&self) {
		let result = rustydialogs::FileDialog {
			title: "Open multiple files from the winit example",
			path: Some(&self.current_dir),
			filters: None,
			owner: self.owner(),
		}.pick_files();
		self.set_status(&format!("open files -> {result:?}"));
	}

	fn save_file(&self) {
		let default_path = self.current_dir.join("winit-output.txt");
		let filters = [
			rustydialogs::FileFilter {
				name: "Text Files",
				patterns: &["*.txt"],
			},
			rustydialogs::FileFilter {
				name: "Markdown Files",
				patterns: &["*.md"],
			},
		];
		let result = rustydialogs::FileDialog {
			title: "Save a file from the winit example",
			path: Some(default_path.as_path()),
			filters: Some(&filters),
			owner: self.owner(),
		}.save_file();
		self.set_status(&format!("save file -> {result:?}"));
	}

	fn choose_folder(&self) {
		let result = rustydialogs::FileDialog {
			title: "Choose a folder from the winit example",
			path: Some(&self.current_dir),
			filters: None,
			owner: self.owner(),
		}.choose_folder();
		self.set_status(&format!("choose folder -> {result:?}"));
	}

	fn choose_folders(&self) {
		let result = rustydialogs::FileDialog {
			title: "Choose multiple folders from the winit example",
			path: Some(&self.current_dir),
			filters: None,
			owner: self.owner(),
		}.choose_folders();
		self.set_status(&format!("choose folders -> {result:?}"));
	}

	fn show_color_picker(&self) {
		let result = rustydialogs::ColorPicker {
			title: "Pick a color from the winit example",
			value: rustydialogs::ColorValue {
				red: 79,
				green: 179,
				blue: 163,
			},
			owner: self.owner(),
		}.show();
		self.set_status(&format!("color picker -> {result:?}"));
	}

	fn show_text_input(&self) {
		let result = rustydialogs::TextInput {
			title: "Text input from the winit example",
			message: "Type anything and press OK.",
			value: "Hello from winit",
			mode: rustydialogs::TextInputMode::SingleLine,
			owner: self.owner(),
		}.show();
		self.set_status(&format!("text input -> {result:?}"));
	}

	fn show_password_input(&self) {
		let result = rustydialogs::TextInput {
			title: "Password input from the winit example",
			message: "Enter a password-like value.",
			value: "",
			mode: rustydialogs::TextInputMode::Password,
			owner: self.owner(),
		}.show();
		self.set_status(&format!("password input -> {result:?}"));
	}

	fn show_multiline_input(&self) {
		let result = rustydialogs::TextInput {
			title: "Multi-line input from the winit example",
			message: "Enter multiple lines, then press OK.",
			value: "Line 1\nLine 2",
			mode: rustydialogs::TextInputMode::MultiLine,
			owner: self.owner(),
		}.show();
		self.set_status(&format!("multi-line input -> {result:?}"));
	}

	fn show_notification(&self) {
		rustydialogs::Notification {
			app_id: APP_ID,
			title: "Rusty Dialogs + winit",
			message: "Notification triggered from the winit integration example.",
			icon: rustydialogs::MessageIcon::Info,
			duration: rustydialogs::NotifyDuration::Short,
		}.show();
		self.set_status("notification requested");
	}

	fn handle_character_key(&self, event_loop: &ActiveEventLoop, key: &str) {
		match key.to_ascii_uppercase().as_str() {
			"H" => self.print_help(),
			"M" => self.show_message_box(),
			"O" => self.open_file(),
			"A" => self.open_files(),
			"S" => self.save_file(),
			"F" => self.choose_folder(),
			"G" => self.choose_folders(),
			"C" => self.show_color_picker(),
			"T" => self.show_text_input(),
			"P" => self.show_password_input(),
			"L" => self.show_multiline_input(),
			"N" => self.show_notification(),
			"Q" => event_loop.exit(),
			_ => {},
		}
	}
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.window.is_some() {
			return;
		}

		self.window = match event_loop.create_window(Window::default_attributes().with_title(BASE_TITLE)) {
			Ok(window) => Some(window),
			Err(err) => {
				eprintln!("failed to create winit window: {err}");
				event_loop.exit();
				return;
			},
		};
		rustydialogs::Notification::setup(APP_ID);
		self.print_help();
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
		if self.window.as_ref().map(|window| window.id()) != Some(window_id) {
			return;
		}

		match event {
			WindowEvent::CloseRequested => event_loop.exit(),
			WindowEvent::KeyboardInput { event, is_synthetic: false, .. } if event.state.is_pressed() => {
				match event.key_without_modifiers().as_ref() {
					Key::Character(key) => self.handle_character_key(event_loop, key),
					Key::Named(NamedKey::Escape) => event_loop.exit(),
					_ => {},
				}
			},
			_ => {},
		}
	}
}
