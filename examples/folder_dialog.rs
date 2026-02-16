fn main() {
	let dialog = rustydialogs::FolderDialog {
		title: "Select a folder",
		directory: None,
	};

	match dialog.show() {
		Some(path) => println!("Selected folder: {}", path.display()),
		None => println!("Folder selection canceled"),
	}
}
