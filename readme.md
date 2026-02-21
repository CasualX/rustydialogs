Rusty Dialogs
=============

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/rustydialogs.svg)](https://crates.io/crates/rustydialogs)
[![docs.rs](https://docs.rs/rustydialogs/badge.svg)](https://docs.rs/rustydialogs)
[![Check](https://github.com/CasualX/rustydialogs/actions/workflows/check.yml/badge.svg)](https://github.com/CasualX/rustydialogs/actions/workflows/check.yml)

Rusty Dialogs is a Rust library that provides a simple and cross-platform way to display native dialog boxes.

Dialogs
-------

The library supports the following types of dialogs:

- MessageBox

  A title, message, icon, and buttons can be specified. The result of the user's choice is returned.

- FileDialog

  A title, directory, file name and filters can be specified. The result of the user's choice is returned.

- TextInput

  A title, message, and default text can be specified. The result of the user's input is returned.

- ColorPicker

  A title and default color can be specified. The result of the user's color selection is returned.

- NotifyPopup

  A title, message, and icon can be specified. The popup is shown as a notification.

Platform Support
----------------

A reasonable effort is made to support the following platforms:

- Windows 7 and later
- Linux with Zenity or KDialog installed
- macOS with `osascript` available

### Windows

On Windows, the library uses the legacy Win32-based common dialogs API to display native dialog boxes.

### Linux

On Linux, `gtk4` and `gtk3` are optional features.
Do not enable both in the same binary: GTK performs a runtime check for mixed major versions and will abort if `gtk3` and `gtk4` are linked together.

When enabled, Rusty Dialogs prefers backends in this order:

1. `gtk4` / `gtk3`
2. `zenity` / `kdialog`

Without `gtk4` and `gtk3`, Rusty Dialogs uses `zenity` or `kdialog` and tries to choose a reasonable program based on the desktop environment.
If neither is available, the library will panic.

You can override backend selection with the `RUSTY_DIALOGS_BACKEND` environment variable.
Supported values are `gtk4`, `gtk3`, `zenity`, and `kdialog`.

### macOS

On macOS, the dialogs are implemented using `osascript` (AppleScript).
Some behaviors are best-effort due to native AppleScript limitations:

- `TextInputMode::MultiLine` currently falls back to a single input dialog.
- Notification timeout is controlled by the OS and may ignore `NotifyPopup::timeout`.

Development
-----------

To check the code on all supported platforms, run the following command:

```bash
cargo check --examples
cargo check --examples --target=x86_64-pc-windows-gnu
cargo check --examples --target=x86_64-unknown-linux-gnu
cargo check --examples --target=aarch64-apple-darwin
```

To test the Windows implementation on Linux, you can use the `wine` compatibility layer:

```bash
cargo build --examples --target=x86_64-pc-windows-gnu
wine target/x86_64-pc-windows-gnu/debug/examples/message_box.exe
```

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

Inspired by [tinyfiledialogs](https://sourceforge.net/projects/tinyfiledialogs/) and [rfd](https://github.com/PolyMeilex/rfd).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
