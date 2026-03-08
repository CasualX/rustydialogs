use std::path::Path;

fn main() {
	let filters = [
		rustydialogs::FileFilter {
			name: "Markdown Files",
			patterns: &["*.md"],
		},
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
		title: "Open file(s)",
		path: Some(Path::new("readme.md")),
		filters: Some(&filters),
		owner: None,
	};

	match dialog.pick_files() {
		Some(paths) => {
			for path in paths {
				println!("Open path: {}", path.display());
			}
		}
		None => println!("Open canceled"),
	}
}
