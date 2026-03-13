use block2::StackBlock;
use objc2::*;
use objc2_app_kit::*;
#[allow(deprecated)] // NSUserNotification is deprecated
use objc2_foundation::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use super::*;

fn run_on_main<R: Send, F: FnOnce(MainThreadMarker) -> R + Send>(run: F) -> R {
	if let Some(mtm) = MainThreadMarker::new() {
		run(mtm)
	}
	else {
		let mtm = unsafe { MainThreadMarker::new_unchecked() };
		let app = NSApplication::sharedApplication(mtm);
		if app.isRunning() {
			dispatch2::run_on_main(run)
		}
		else {
			panic!("cannot show AppKit dialogs from a non-main thread before NSApplication is running");
		}
	}
}

struct PolicyManager {
	app: rc::Retained<NSApplication>,
	initial_policy: NSApplicationActivationPolicy,
}

impl PolicyManager {
	fn new(mtm: MainThreadMarker) -> PolicyManager {
		let app = NSApplication::sharedApplication(mtm);
		let initial_policy = app.activationPolicy();

		if initial_policy == NSApplicationActivationPolicy::Prohibited {
			app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
		}

		PolicyManager { app, initial_policy }
	}
}

impl Drop for PolicyManager {
	fn drop(&mut self) {
		self.app.setActivationPolicy(self.initial_policy);
	}
}

struct FocusManager {
	key_window: Option<rc::Retained<NSWindow>>,
}

impl FocusManager {
	fn new(mtm: MainThreadMarker) -> FocusManager {
		let app = NSApplication::sharedApplication(mtm);
		let key_window = app.keyWindow();
		FocusManager { key_window }
	}
}

impl Drop for FocusManager {
	fn drop(&mut self) {
		if let Some(window) = &self.key_window {
			window.makeKeyAndOrderFront(None);
		}
	}
}

fn owner_handle(owner: Option<&dyn HasWindowHandle>) -> Option<usize> {
	let raw = owner.and_then(|window| window.window_handle().ok()).map(|handle| handle.as_raw());
	match raw {
		Some(RawWindowHandle::AppKit(handle)) => Some(handle.ns_view.as_ptr() as usize),
		_ => None,
	}
}

fn window_from_view_ptr(view: usize) -> Option<rc::Retained<NSWindow>> {
	let view = view as *mut NSView;
	let view = unsafe { rc::Retained::retain(view) }?;
	view.window()
}

fn owner_window(owner: Option<usize>) -> Option<rc::Retained<NSWindow>> {
	owner.and_then(window_from_view_ptr)
}

fn begin_alert_sheet(alert: &NSAlert, owner: Option<&NSWindow>, mtm: MainThreadMarker) {
	if let Some(owner) = owner {
		let completion = StackBlock::new(move |response| {
			NSApplication::sharedApplication(mtm).stopModalWithCode(response);
		});
		alert.beginSheetModalForWindow_completionHandler(owner, Some(&*completion));
	}
}

fn begin_panel_sheet(panel: &NSSavePanel, owner: Option<&NSWindow>) {
	if let Some(owner) = owner {
		let completion = StackBlock::new(|_: isize| {});
		panel.beginSheetModalForWindow_completionHandler(owner, &*completion);
	}
}

fn apply_filters(panel: &NSSavePanel, filters: Option<&[FileFilter<'_>]>) {
	let Some(file_types) = filter_types(filters) else {
		return;
	};

	let file_types = file_types.iter().map(|value| NSString::from_str(value)).collect::<Vec<_>>();
	let array = NSArray::from_retained_slice(&file_types);

	#[allow(deprecated)]
	panel.setAllowedFileTypes(Some(&array));
}

fn filter_types(filters: Option<&[FileFilter<'_>]>) -> Option<Vec<String>> {
	let filters = filters?;
	let mut result = Vec::new();

	for filter in filters {
		for pattern in filter.patterns {
			let Some(file_type) = pattern_to_file_type(pattern) else {
				return None;
			};
			if !result.iter().any(|item| item == &file_type) {
				result.push(file_type);
			}
		}
	}

	if result.is_empty() { None } else { Some(result) }
}

fn pattern_to_file_type(pattern: &str) -> Option<String> {
	let pattern = pattern.trim();
	if pattern.is_empty() || pattern == "*" || pattern == "*.*" {
		return None;
	}

	let file_type = pattern.strip_prefix("*.")
		.or_else(|| pattern.strip_prefix('.'))
		.unwrap_or(pattern);

	if file_type.is_empty()
		|| file_type.contains('*')
		|| file_type.contains('?')
		|| file_type.contains('/')
		|| file_type.contains('\\')
	{
		return None;
	}

	Some(file_type.to_string())
}

pub fn message_box(p: &MessageBox<'_>) -> Option<MessageResult> {
	let title_text = p.title;
	let message_text = p.message;
	let icon = p.icon;
	let buttons = p.buttons;
	let owner = owner_handle(p.owner);

	run_on_main(move |mtm| {
		let _policy_manager = PolicyManager::new(mtm);
		let _focus_manager = FocusManager::new(mtm);
		let owner = owner_window(owner);
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

		begin_alert_sheet(&alert, owner.as_deref(), mtm);
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
	let filters = p.filters;
	let owner = owner_handle(p.owner);

	run_on_main(move |mtm| {
		let _policy_manager = PolicyManager::new(mtm);
		let _focus_manager = FocusManager::new(mtm);
		let owner = owner_window(owner);
		let panel = NSSavePanel::savePanel(mtm);
		let title = NSString::from_str(title);
		panel.setTitle(Some(&title));
		panel.setCanCreateDirectories(true);
		apply_filters(&panel, filters);

		let (directory, default_name) = initial_directory_and_name(path);
		if let Some(directory) = directory {
			let dir_url = path_to_file_url(&directory);
			panel.setDirectoryURL(Some(&dir_url));
		}

		if let Some(default_name) = default_name {
			let name = NSString::from_str(&default_name);
			panel.setNameFieldStringValue(&name);
		}

		begin_panel_sheet(&panel, owner.as_deref());
		let response = panel.runModal();
		if response != NSModalResponseOK {
			return None;
		}

		panel.URL().and_then(url_into_pathbuf)
	})
}

pub fn choose_folder(p: &FileDialog<'_>) -> Option<PathBuf> {
	choose_folders_impl(p, false).and_then(|paths| paths.into_iter().next())
}

pub fn choose_folders(p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	choose_folders_impl(p, true)
}

fn choose_folders_impl(p: &FileDialog<'_>, multiple: bool) -> Option<Vec<PathBuf>> {
	let title_text = p.title;
	let directory = p.path;
	let owner = owner_handle(p.owner);

	run_on_main(move |mtm| {
		let _policy_manager = PolicyManager::new(mtm);
		let _focus_manager = FocusManager::new(mtm);
		let owner = owner_window(owner);
		let panel = NSOpenPanel::openPanel(mtm);
		let title = NSString::from_str(title_text);
		panel.setTitle(Some(&title));
		panel.setCanChooseDirectories(true);
		panel.setCanChooseFiles(false);
		panel.setAllowsMultipleSelection(multiple);
		panel.setCanCreateDirectories(true);

		if let Some(directory) = directory {
			if let Some(path) = utils::abspath(Some(directory)) {
				let dir_url = path_to_file_url(path.as_ref());
				panel.setDirectoryURL(Some(&dir_url));
			}
		}

		begin_panel_sheet(&panel, owner.as_deref());
		let response = panel.runModal();
		if response != NSModalResponseOK {
			return None;
		}

		let paths = if multiple {
			panel.URLs().iter().filter_map(url_into_pathbuf).collect::<Vec<_>>()
		}
		else {
			panel.URL().and_then(url_into_pathbuf).into_iter().collect::<Vec<_>>()
		};
		if paths.is_empty() { None } else { Some(paths) }
	})
}

pub fn text_input(p: &TextInput<'_>) -> Option<String> {
	let title_text = p.title;
	let message_text = p.message;
	let value_text = p.value;
	let mode = p.mode;
	let owner = owner_handle(p.owner);

	run_on_main(move |mtm| {
		let _policy_manager = PolicyManager::new(mtm);
		let _focus_manager = FocusManager::new(mtm);
		let owner = owner_window(owner);
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

				begin_alert_sheet(&alert, owner.as_deref(), mtm);
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

				begin_alert_sheet(&alert, owner.as_deref(), mtm);
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

				begin_alert_sheet(&alert, owner.as_deref(), mtm);
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
	let owner = owner_handle(p.owner);

	run_on_main(move |mtm| {
		let _policy_manager = PolicyManager::new(mtm);
		let _focus_manager = FocusManager::new(mtm);
		let owner = owner_window(owner);
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

		begin_alert_sheet(&alert, owner.as_deref(), mtm);
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
	let filters = p.filters;
	let owner = owner_handle(p.owner);

	run_on_main(move |mtm| {
		let _policy_manager = PolicyManager::new(mtm);
		let _focus_manager = FocusManager::new(mtm);
		let owner = owner_window(owner);
		let panel = NSOpenPanel::openPanel(mtm);
		let title = NSString::from_str(title_text);
		panel.setTitle(Some(&title));
		panel.setCanChooseDirectories(false);
		panel.setCanChooseFiles(true);
		panel.setAllowsMultipleSelection(multiple);
		panel.setCanCreateDirectories(true);
		apply_filters(&panel, filters);

		if let Some(initial_path) = initial_path {
			let directory = initial_directory(Some(initial_path));
			if let Some(directory) = directory {
				let dir_url = path_to_file_url(&directory);
				panel.setDirectoryURL(Some(&dir_url));
			}
		}

		begin_panel_sheet(&panel, owner.as_deref());
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

fn path_to_file_url(path: &Path) -> rc::Retained<NSURL> {
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

fn color_value_to_nscolor(color: ColorValue) -> rc::Retained<NSColor> {
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

fn url_into_pathbuf(url: rc::Retained<NSURL>) -> Option<PathBuf> {
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
