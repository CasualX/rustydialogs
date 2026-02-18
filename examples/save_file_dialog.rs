use std::path::Path;

fn main() {
	let directory = std::env::current_dir().ok();

	let filters = [
		rustydialogs::FileFilter {
			desc: "Text files (*.txt)",
			patterns: &["*.txt"],
		},
		rustydialogs::FileFilter {
			desc: "JSON files (*.json)",
			patterns: &["*.json"],
		},
	];

	let dialog = rustydialogs::FileDialog {
		title: "Save a file",
		directory: directory.as_deref(),
		file_name: Some(Path::new("output.txt")),
		filter: Some(&filters),
		owner: None,
	};

	match dialog.save_file() {
		Some(path) => println!("Save path: {}", path.display()),
		None => println!("Save canceled"),
	}
}
