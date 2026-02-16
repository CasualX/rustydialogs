fn main() {
	let input = rustydialogs::TextInput {
		title: "TextInput",
		message: "Enter some text:",
		value: "default value",
		mode: rustydialogs::TextInputMode::Multi,
	};

	match input.show() {
		Some(value) => println!("TextInput: {value}"),
		None => println!("Canceled"),
	}
}
