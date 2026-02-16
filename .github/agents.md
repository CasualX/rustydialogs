# Copilot Instructions for rustydialogs

## Big Picture
- `rustydialogs` is a small cross-platform dialog abstraction crate: public API in `src/lib.rs`, platform backends in `src/linux/` and `src/win32/`.
- `src/lib.rs` defines dialog data types (`MessageBox`, `FileDialog`, `FolderDialog`, `TextInput`, `ColorPicker`, `NotifyPopup`) and thin `show`/`pick_*` methods that delegate to platform functions.
- Linux uses external executables (`zenity`/`kdialog`) selected once via `LazyLock` in `src/linux/mod.rs`.
- Windows uses native Win32 APIs through the `windows` crate; backend dispatch lives in `src/win32/mod.rs`.

## Architectural Pattern to Follow
- When adding/changing a dialog type, keep a **three-layer flow**:
  1. Public struct + method in `src/lib.rs`
  2. Platform dispatcher in `src/linux/mod.rs` and `src/win32/mod.rs`
  3. Concrete backend implementation files (`src/linux/zenity.rs`, `src/linux/kdialog.rs`, `src/win32/*.rs`)
- Keep backend functions named consistently (`show`, `pick_file`, `pick_files`, `save_file`, `folder_dialog`, `text_input`, etc.).
- Prefer focused modules by concern on Windows (`file.rs`, `folder.rs`, `message.rs`, `input.rs`, `color.rs`, `notify.rs`).

## Linux Backend Conventions
- Build command args as `&OsStr` using helper `s(...)` from `src/linux/mod.rs`.
- Use shared invocation helpers from `src/linux/mod.rs` (`invoke`, `invoke_output`, `invoke_output_bytes`, `invoke_async`) rather than ad-hoc `Command` code.
- Treat non-success exit codes as cancel (`None`) unless the dialog semantics require special handling.
- Parse command stdout as bytes for filesystem paths (`OsStrExt::from_bytes`) to avoid UTF-8 assumptions.
- Respect backend selection env var: `RUSTY_DIALOGS_BACKEND=zenity|kdialog`.

## Windows Backend Conventions
- Use UTF-16 conversion helper `utf16_null_terminated(...)` from `src/win32/mod.rs`.
- Keep Win32 calls in small, dedicated modules; return `Option<...>` where user cancel maps to `None`.
- For file/folder pickers, preserve current style: prepare buffers/structs (`OPENFILENAMEW`, `BROWSEINFOW`), call API, parse null-terminated output.
- If you add new Windows APIs, update feature flags in `Cargo.toml` under `target.'cfg(windows)'.dependencies.windows.features`.

## Developer Workflows
- Main validation command: `cargo check --examples`.
- Cross-target checks used in this repo:
  - `cargo check --examples --target=x86_64-pc-windows-gnu`
  - `cargo check --examples --target=x86_64-unknown-linux-gnu`
- Example programs in `examples/` are the primary manual smoke tests; `examples/run_all.sh` runs them sequentially.
- For Windows behavior from Linux, README documents using Wine after `cargo build --examples --target=x86_64-pc-windows-gnu`.

## Practical Guardrails for Edits
- Keep public API minimal and data-oriented (plain structs + enums, no builder pattern currently).
- Match existing formatting/style (tabs, concise helper functions, `Option`-based cancel semantics).
- Avoid introducing extra dependencies unless required by platform API usage.
- Update/add an example in `examples/` when adding a new dialog capability.
