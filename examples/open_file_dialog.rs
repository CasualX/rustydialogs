use std::path::Path;

fn main() {
	let filters = [
		rustydialogs::FileFilter {
			desc: "Markdown Files (*.md)",
			patterns: &["*.md"],
		},
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
		title: "Open file(s)",
		path: Some(Path::new("readme.md")),
		filter: Some(&filters),
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
