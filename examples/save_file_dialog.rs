use std::path::Path;

fn main() {
	let filters = [
		rustydialogs::FileFilter {
			name: "Text Files",
			patterns: &["*.txt"],
		},
		rustydialogs::FileFilter {
			name: "JSON Files",
			patterns: &["*.json"],
		},
	];

	let dialog = rustydialogs::FileDialog {
		title: "Save a file",
		path: Some(Path::new("output.txt")),
		filters: Some(&filters),
		owner: None,
	};

	match dialog.save_file() {
		Some(path) => println!("Save path: {}", path.display()),
		None => println!("Save canceled"),
	}
}
