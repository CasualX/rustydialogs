fn main() {
	let current_dir = std::env::current_dir().ok();

	let dialog = rustydialogs::FileDialog {
		title: "Select folders",
		path: current_dir.as_deref(),
		filters: None,
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
