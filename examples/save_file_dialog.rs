use std::path::Path;

fn main() {
	let filters = [
		rustydialogs::FileFilter {
			desc: "Text Files (*.txt)",
			patterns: &["*.txt"],
		},
		rustydialogs::FileFilter {
			desc: "JSON Files (*.json)",
			patterns: &["*.json"],
		},
	];

	let dialog = rustydialogs::FileDialog {
		title: "Save a file",
		path: Some(Path::new("output.txt")),
		filter: Some(&filters),
		owner: None,
	};

	match dialog.save_file() {
		Some(path) => println!("Save path: {}", path.display()),
		None => println!("Save canceled"),
	}
}
