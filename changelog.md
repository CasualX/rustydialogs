# Changelog

## 0.5.0

### Breaking changes

- Requires Rust 1.95 or later; the crate now uses edition 2024.
- Replaces `FolderDialog` with `FileDialog::choose_folder()` and
  `FileDialog::choose_folders()`. Use `path` for the initial directory and
  `filters: None` for folder selection.
- Renames `FileDialog::filter` to `filters` and `FileFilter::desc` to `name`.
- Replaces `Notification::timeout` and the `SHORT_TIMEOUT`/`LONG_TIMEOUT`
  constants with `duration: NotifyDuration::{Short, Long, Infinite}`.
- `Notification::setup()` now returns `bool` to indicate setup success.

### Improvements

- Adds multiple-folder selection, with a single-folder fallback for KDialog.
- Improves macOS AppleScript and AppKit dialogs, including file selection,
  filters, cancellation, text/path handling, owner sheets, and notifications.
- Adds a winit integration example and macOS test reports documenting backend
  behavior and limitations.
- Expands CI compile checks for platform backends and the minimum Rust version.
