fn main() {
	let picker = rustydialogs::ColorPicker {
		title: "Pick a color",
		value: rustydialogs::ColorValue {
			red: 0xFF,
			green: 0x00,
			blue: 0x77,
		},
		owner: None,
	};

	match picker.show() {
		Some(color) => println!(
			"Selected color: #{:02X}{:02X}{:02X}",
			color.red, color.green, color.blue
		),
		None => println!("Color selection canceled"),
	}
}
