Test Report
===========

Notes for each backend running the interactive tests example. See `examples/tests.rs` for details.

The detailed output of each test run can be found in the [testreport](testreport) folder.

Host: Linux
-----------

### Zenity

```bash
RUSTY_DIALOGS_BACKEND=zenity cargo run --example tests
```

- _MessageBox_: Dismissing the dialog by pressing ESC returns the Cancel/No response instead of None.

- _MessageBox_: Question dialogs reuse Info icons because Zenity has odd limitations in that regard.

- _TextInput_: MultiLine input doesn't display the message, only the title.
  SingleLine and Password input works fine.

### KDialog

```bash
RUSTY_DIALOGS_BACKEND=kdialog cargo run --example tests
```

- _MessageBox_: Dismissing the dialog by pressing ESC returns `Some(Cancel)` when using `YesNoCancel` buttons and `None` otherwise.

### GTK3

```bash
RUSTY_DIALOGS_BACKEND=gtk3 cargo run --example tests --features gtk3
```

- _MessageBox_: General styling of the dialog is a bit off. Title could be bigger.

### GTK4

```bash
RUSTY_DIALOGS_BACKEND=gtk4 cargo run --example tests --features gtk4
```

- GTK-Message: `GtkDialog mapped without a transient parent. This is discouraged.`

- _MessageBox_: No message box icon is shown.

- _FileDialog_: When selecting multiple files, the order of the returned paths is not the order in which they were selected.

- _FileDialog_: GTK-WARNING: `Attempting to add '...' to the list of recently used resources, but no name of the application that is registering it was defined`

### XDG Portal

```bash
RUSTY_DIALOGS_BACKEND=xdg-portal cargo run --example tests --features xdg-portal
```

- _MessageBox_: Not available. Always returns `None`.

- _FileDialog_: When selecting multiple files, the order of the returned paths is not the order in which they were selected.

- _ColorPicker_: Not available. Always returns `None`.

- _TextInput_: Not available. Always returns `None`.

- _Notification_: Doesn't work. No notification appears.

### Windows under Wine

```bash
cargo build --examples --target=x86_64-pc-windows-gnu
wine ./target/x86_64-pc-windows-gnu/debug/examples/tests.exe
```

- _MessageBox_: Pressing ESC always returns `Some(Cancel)` regardless of the buttons used.

- _FolderDialog_: The starting directory doesn't really work well.

- _Notification_: Tray Icon based notifications work. HTA and WinRT Toast notifications are not supported.

Host: Windows
-------------

```cmd
cargo run --example tests
```

- _MessageBox_: Pressing ESC returns the Cancel/No response instead of None.

- _MessageBox_: YesNo dialogs cannot be dismissed by pressing ESC.

- _FolderDialog_: The starting directory doesn't really work well.

### WinRT Toast Notifications

```cmd
cargo run --example tests --features winrt-toast -- n
```

- _Notification_: The first notifications after setup are not shown because it takes some time for the app to be registered in the system. After that, notifications work fine.
