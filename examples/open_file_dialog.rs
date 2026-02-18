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
		title: "Open file(s)",
		directory: directory.as_deref(),
		file_name: None,
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
