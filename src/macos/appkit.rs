use dispatch2::run_on_main;
use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_app_kit::{
	NSAlert,
	NSAlertFirstButtonReturn,
	NSColor,
	NSColorSpace,
	NSColorWell,
	NSColorWellStyle,
	NSAlertSecondButtonReturn,
	NSAlertStyle,
	NSAlertThirdButtonReturn,
	NSModalResponseOK,
	NSOpenPanel,
	NSScrollView,
	NSSavePanel,
	NSSecureTextField,
	NSTextField,
	NSTextView,
	NSView,
};
#[allow(deprecated)] // NSUserNotification is deprecated
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString, NSURL, NSUserNotification, NSUserNotificationCenter};

use super::*;

pub fn message_box(p: &MessageBox<'_>) -> Option<MessageResult> {
	let title_text = p.title;
	let message_text = p.message;
	let icon = p.icon;
	let buttons = p.buttons;

	run_on_main(move |mtm| {
		let alert = NSAlert::new(mtm);
		let title = NSString::from_str(title_text);
		let message = NSString::from_str(message_text);

		alert.setMessageText(&title);
		alert.setInformativeText(&message);
		alert.setAlertStyle(match icon {
			MessageIcon::Info | MessageIcon::Question => NSAlertStyle::Informational,
			MessageIcon::Warning => NSAlertStyle::Warning,
			MessageIcon::Error => NSAlertStyle::Critical,
		});

		let labels: &[&str] = match buttons {
			MessageButtons::Ok => &["OK"],
			MessageButtons::OkCancel => &["OK", "Cancel"],
			MessageButtons::YesNo => &["Yes", "No"],
			MessageButtons::YesNoCancel => &["Yes", "No", "Cancel"],
		};
		for label in labels {
			alert.addButtonWithTitle(&NSString::from_str(label));
		}

		let response = alert.runModal();

		let results: &[MessageResult] = match buttons {
			MessageButtons::Ok => &[MessageResult::Ok],
			MessageButtons::OkCancel => &[MessageResult::Ok, MessageResult::Cancel],
			MessageButtons::YesNo => &[MessageResult::Yes, MessageResult::No],
			MessageButtons::YesNoCancel => &[MessageResult::Yes, MessageResult::No, MessageResult::Cancel],
		};

		let index = if response == NSAlertFirstButtonReturn { 0 }
		else if response == NSAlertSecondButtonReturn { 1 }
		else if response == NSAlertThirdButtonReturn { 2 }
		else { !0 }; // Out of bounds, will be handled below

		results.get(index).copied()
	})
}

pub fn pick_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	run_open_panel(p, false).and_then(|paths| paths.into_iter().next())
}

pub fn pick_files(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	run_open_panel(p, true)
}

pub fn save_file(p: &FileDialog<'_>) -> Option<PathBuf> {
	let title = p.title;
	let path = p.path;

	run_on_main(move |mtm| {
		let panel = NSSavePanel::savePanel(mtm);
		let title = NSString::from_str(title);
		panel.setTitle(Some(&title));
		panel.setCanCreateDirectories(true);

		let (directory, default_name) = initial_directory_and_name(path);
		if let Some(directory) = directory {
			let dir_url = path_to_file_url(&directory);
			panel.setDirectoryURL(Some(&dir_url));
		}

		if let Some(default_name) = default_name {
			let name = NSString::from_str(&default_name);
			panel.setNameFieldStringValue(&name);
		}

		let response = panel.runModal();
		if response != NSModalResponseOK {
			return None;
		}

		panel.URL().and_then(url_into_pathbuf)
	})
}

pub fn folder_dialog(p: &FolderDialog<'_>) -> Option<PathBuf> {
	let title_text = p.title;
	let directory = p.directory;

	run_on_main(move |mtm| {
		let panel = NSOpenPanel::openPanel(mtm);
		let title = NSString::from_str(title_text);
		panel.setTitle(Some(&title));
		panel.setCanChooseDirectories(true);
		panel.setCanChooseFiles(false);
		panel.setAllowsMultipleSelection(false);
		panel.setCanCreateDirectories(true);

		if let Some(directory) = directory {
			if let Some(path) = utils::abspath(Some(directory)) {
				let dir_url = path_to_file_url(path.as_ref());
				panel.setDirectoryURL(Some(&dir_url));
			}
		}

		let response = panel.runModal();
		if response != NSModalResponseOK {
			return None;
		}

		panel.URL().and_then(url_into_pathbuf)
	})
}

pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	let title_text = p.title;
	let message_text = p.message;
	let value_text = p.value;
	let mode = p.mode;

	run_on_main(move |mtm| {
		let alert = NSAlert::new(mtm);
		let title = NSString::from_str(title_text);
		let message = NSString::from_str(message_text);
		let ok = NSString::from_str("OK");
		let cancel = NSString::from_str("Cancel");

		alert.setMessageText(&title);
		alert.setInformativeText(&message);
		alert.addButtonWithTitle(&ok);
		alert.addButtonWithTitle(&cancel);

		match mode {
			TextInputMode::Password => {
				let frame = text_field_frame();
				let field = NSSecureTextField::initWithFrame(NSSecureTextField::alloc(mtm), frame);
				let value = NSString::from_str(value_text);
				field.setStringValue(&value);
				alert.setAccessoryView(Some(&field));

				let response = alert.runModal();
				if response != NSAlertFirstButtonReturn { None }
				else { Some(field.stringValue().to_string()) }
			}
			TextInputMode::MultiLine => {
				let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(380.0, 160.0));
				let scroll = NSScrollView::initWithFrame(NSScrollView::alloc(mtm), frame);
				scroll.setHasVerticalScroller(true);
				scroll.setHasHorizontalScroller(false);

				let field = NSTextView::initWithFrame(NSTextView::alloc(mtm), frame);
				field.setEditable(true);
				field.setSelectable(true);
				let value = NSString::from_str(value_text);
				field.setString(&value);

				scroll.setDocumentView(Some(&field));
				alert.setAccessoryView(Some(&scroll));

				let response = alert.runModal();
				if response != NSAlertFirstButtonReturn { None }
				else { Some(field.string().to_string()) }
			}
			TextInputMode::SingleLine => {
				let frame = text_field_frame();
				let value = NSString::from_str(value_text);
				let field = NSTextField::initWithFrame(NSTextField::alloc(mtm), frame);
				field.setStringValue(&value);
				alert.setAccessoryView(Some(&field));

				let response = alert.runModal();
				if response != NSAlertFirstButtonReturn { None }
				else { Some(field.stringValue().to_string()) }
			}
		}
	})
}

pub fn color_picker(p: &ColorPicker<'_>) -> Option<ColorValue> {
	let title_text = p.title;
	let initial = p.value;

	run_on_main(|mtm| {
		let alert = NSAlert::new(mtm);
		let title = NSString::from_str(title_text);
		let ok = NSString::from_str("OK");
		let cancel = NSString::from_str("Cancel");

		alert.setMessageText(&title);
		alert.addButtonWithTitle(&ok);
		alert.addButtonWithTitle(&cancel);

		let container = NSView::initWithFrame(NSView::alloc(mtm), color_well_container_frame());
		let well = NSColorWell::initWithFrame(NSColorWell::alloc(mtm), color_well_frame());
		let _ = NSColorWellStyle::Default;
		let initial = color_value_to_nscolor(initial);
		well.setColor(&initial);
		container.addSubview(&well);
		alert.setAccessoryView(Some(&container));

		let response = alert.runModal();
		if response != NSAlertFirstButtonReturn {
			return None;
		}

		nscolor_to_color_value(&well.color())
	})
}

#[inline]
pub fn notify_setup(_app_id: &str) -> bool {
	// No explicit setup required for NSUserNotificationCenter.
	true
}

#[allow(deprecated)]
pub fn notify(p: &Notification<'_>) {
	if !notify_setup(p.app_id) {
		return;
	}

	run_on_main(|_mtm| {
		let center = NSUserNotificationCenter::defaultUserNotificationCenter();
		let notification = NSUserNotification::new();

		let title = NSString::from_str(p.title);
		let message = NSString::from_str(p.message);

		notification.setTitle(Some(&title));
		notification.setInformativeText(Some(&message));

		if !p.app_id.is_empty() {
			let subtitle = NSString::from_str(p.app_id);
			notification.setSubtitle(Some(&subtitle));
		}

		center.deliverNotification(&notification);
	});
}

fn run_open_panel(p: &FileDialog<'_>, multiple: bool) -> Option<Vec<PathBuf>> {
	let title_text = p.title;
	let initial_path = p.path;

	run_on_main(move |mtm| {
		let panel = NSOpenPanel::openPanel(mtm);
		let title = NSString::from_str(title_text);
		panel.setTitle(Some(&title));
		panel.setCanChooseDirectories(false);
		panel.setCanChooseFiles(true);
		panel.setAllowsMultipleSelection(multiple);
		panel.setCanCreateDirectories(true);

		if let Some(initial_path) = initial_path {
			let directory = initial_directory(Some(initial_path));
			if let Some(directory) = directory {
				let dir_url = path_to_file_url(&directory);
				panel.setDirectoryURL(Some(&dir_url));
			}
		}

		let response = panel.runModal();
		if response != NSModalResponseOK {
			return None;
		}

		if multiple {
			let files = panel.URLs().iter().filter_map(url_into_pathbuf).collect();
			Some(files)
		}
		else {
			panel.URL().and_then(url_into_pathbuf).map(|path| vec![path])
		}
	})
}

fn path_to_file_url(path: &Path) -> Retained<NSURL> {
	let path = NSString::from_str(&path.to_string_lossy());
	NSURL::fileURLWithPath(&path)
}

fn text_field_frame() -> NSRect {
	NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(320.0, 24.0))
}

fn color_well_container_frame() -> NSRect {
	NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(320.0, 32.0))
}

fn color_well_frame() -> NSRect {
	NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(120.0, 28.0))
}

fn color_value_to_nscolor(color: ColorValue) -> Retained<NSColor> {
	NSColor::colorWithSRGBRed_green_blue_alpha(
		u8_to_component(color.red),
		u8_to_component(color.green),
		u8_to_component(color.blue),
		1.0,
	)
}

fn nscolor_to_color_value(color: &NSColor) -> Option<ColorValue> {
	let color_space = NSColorSpace::sRGBColorSpace();
	let color = color.colorUsingColorSpace(&color_space)?;

	Some(ColorValue {
		red: component_to_u8(color.redComponent()),
		green: component_to_u8(color.greenComponent()),
		blue: component_to_u8(color.blueComponent()),
	})
}

fn u8_to_component(value: u8) -> f64 {
	(value as f64) / 255.0
}

fn component_to_u8(value: f64) -> u8 {
	(value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn url_into_pathbuf(url: Retained<NSURL>) -> Option<PathBuf> {
	let path = url.path()?;
	Some(PathBuf::from(path.to_string()))
}

fn initial_directory(initial_path: Option<&Path>) -> Option<PathBuf> {
	let (directory, _) = initial_directory_and_name(initial_path);
	directory
}

fn initial_directory_and_name(initial_path: Option<&Path>) -> (Option<PathBuf>, Option<String>) {
	let Some(path) = utils::abspath(initial_path) else {
		return (None, None);
	};

	if path.is_dir() {
		return (Some(path.into_owned()), None);
	}

	let directory = path.parent().map(Path::to_path_buf);
	let file_name = path.file_name().and_then(|name| name.to_str()).map(str::to_string);
	(directory, file_name)
}
