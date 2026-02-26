fn main() {
	let current_dir = std::env::current_dir().ok();

	let dialog = rustydialogs::FolderDialog {
		title: "Select a folder",
		directory: current_dir.as_deref(),
		owner: None,
	};

	match dialog.show() {
		Some(path) => println!("Selected folder: {}", path.display()),
		None => println!("Folder selection canceled"),
	}
}
