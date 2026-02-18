fn main() {
	let mode = match std::process::id() / 4 % 3 {
		0 => rustydialogs::TextInputMode::SingleLine,
		1 => rustydialogs::TextInputMode::MultiLine,
		_ => rustydialogs::TextInputMode::Password,
	};

	let input = rustydialogs::TextInput {
		title: "TextInput",
		message: "Enter some text:",
		value: "default value",
		mode,
		owner: None,
	};

	match input.show() {
		Some(value) => println!("TextInput: {value}"),
		None => println!("Canceled"),
	}
}
