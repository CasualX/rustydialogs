use super::*;

#[inline]
pub fn message_box(_p: &MessageBox<'_>) -> Option<MessageResult> {
	None
}

#[inline]
pub fn pick_file(_p: &FileDialog<'_>) -> Option<PathBuf> {
	None
}

#[inline]
pub fn pick_files(_p: &FileDialog<'_>) -> Option<Vec<PathBuf>> {
	None
}

#[inline]
pub fn save_file(_p: &FileDialog<'_>) -> Option<PathBuf> {
	None
}

#[inline]
pub fn folder_dialog(_p: &FolderDialog<'_>) -> Option<PathBuf> {
	None
}

#[inline]
pub fn choose_folders(_p: &FolderDialog<'_>) -> Option<Vec<PathBuf>> {
	None
}

#[inline]
pub fn text_input(_p: &TextInput<'_>) -> Option<String> {
	None
}

#[inline]
pub fn color_picker(_p: &ColorPicker<'_>) -> Option<ColorValue> {
	None
}

#[inline]
pub fn notify_setup(_app_id: &str) -> bool {
	false
}

#[inline]
pub fn notify(_p: &Notification<'_>) {
}
