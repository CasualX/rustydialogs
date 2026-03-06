fn main() {
	let current_dir = std::env::current_dir().ok();

	let dialog = rustydialogs::FolderDialog {
		title: "Select folders",
		directory: current_dir.as_deref(),
		owner: None,
	};

	match dialog.choose_folders() {
		Some(paths) => {
			println!("Selected folders:");
			for path in paths {
				println!("- {}", path.display());
			}
		}
		None => println!("Folder selection canceled"),
	}
}
