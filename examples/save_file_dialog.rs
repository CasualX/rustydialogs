use std::path::Path;

fn main() {
	let filters = [
		rustydialogs::FileFilter {
			desc: "Text files",
			patterns: &["*.txt"],
		},
		rustydialogs::FileFilter {
			desc: "JSON files",
			patterns: &["*.json"],
		},
	];

	let dialog = rustydialogs::FileDialog {
		title: "Save a file",
		directory: None,
		file_name: Some(Path::new("output.txt")),
		filter: Some(&filters),
	};

	match dialog.save_file() {
		Some(path) => println!("Save path: {}", path.display()),
		None => println!("Save canceled"),
	}
}
