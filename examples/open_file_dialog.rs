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
		title: "Open file(s)",
		directory: None,
		file_name: None,
		filter: Some(&filters),
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
