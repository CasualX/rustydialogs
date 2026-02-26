# Copilot Instructions for rustydialogs

## Big Picture
- `rustydialogs` is a small cross-platform dialog abstraction crate: public API in `src/lib.rs`, platform backends in `src/linux/`, `src/windows/`, and `src/macos/`.
- `src/lib.rs` defines dialog data types (`MessageBox`, `FileDialog`, `FolderDialog`, `TextInput`, `ColorPicker`, `NotifyPopup`) and thin `show`/`pick_*` methods that delegate to platform functions.
- Linux backend selection happens once in `src/linux/mod.rs`.
- Windows uses native Win32 APIs through the `windows` crate; backend dispatch and concrete implementation lives in `src/windows/`.
- macOS uses `osascript`-based implementations in `src/macos/mod.rs`.

## Architectural Pattern to Follow
- When adding/changing a dialog type, keep a **three-layer flow**:
  1. Public struct + method in `src/lib.rs`
  2. Platform dispatcher in each platform module (`src/linux/mod.rs`, `src/windows/mod.rs`, `src/macos/mod.rs`)
  3. Concrete backend implementation files (`src/linux/*.rs`, `src/windows/*.rs`, and macOS logic in `src/macos/mod.rs`)
- Keep backend functions named consistently (`show`, `pick_file`, `pick_files`, `save_file`, `folder_dialog`, `text_input`, etc.).
- Prefer focused modules by concern on Windows (`src/windows/file.rs`, `folder.rs`, `message.rs`, `input.rs`, `color.rs`, `notify.rs`).

## Linux Backend Conventions
- Build command args as `&OsStr` using helper `os(...)` from `src/linux/mod.rs`.
- Use shared invocation helpers from `src/linux/mod.rs` (`invoke`, `invoke_output`, `invoke_output_bytes`, `invoke_async`) rather than ad-hoc `Command` code.
- Treat non-success exit codes as cancel (`None`) unless the dialog semantics require special handling.
- Parse command stdout as bytes for filesystem paths (`OsStrExt::from_bytes`) to avoid UTF-8 assumptions.
- Respect backend selection env var: `RUSTY_DIALOGS_BACKEND=gtk4|gtk3|xdg-portal|zenity|kdialog` (feature-dependent).

## Linux Backend Notes
- Feature backends exist in addition to executable-based backends:
  - `gtk4` backend in `src/linux/gtk4/`
  - `gtk3` backend in `src/linux/gtk3/`
  - `xdg-portal` backend in `src/linux/xdg_portal.rs` (currently partial)
- If `gtk4` or `gtk3` feature is enabled, backend selection currently prefers GTK first.

## Windows Backend Conventions
- Use UTF-16 conversion helper `utf16cs(...)` from `src/windows/mod.rs`.
- Keep Win32 calls in small, dedicated modules; return `Option<...>` where user cancel maps to `None`.
- For file/folder pickers, preserve current style: prepare buffers/structs (`OPENFILENAMEW`, `BROWSEINFOW`), call API, parse null-terminated output.
- If you add new Windows APIs, update feature flags in `Cargo.toml` under `target.'cfg(windows)'.dependencies.windows.features`.

## macOS Backend Conventions
- Keep implementation centralized in `src/macos/mod.rs` unless complexity justifies splitting.
- Use existing `osascript` invocation helpers/patterns; maintain `Option`-based cancel semantics.
- Preserve documented best-effort behavior (e.g., multiline text input and notification timeout caveats).

## Developer Workflows
- Main validation command: `cargo check --examples`.
- Cross-target checks used in this repo:
  - `cargo check --examples --target=x86_64-pc-windows-gnu`
  - `cargo check --examples --target=x86_64-unknown-linux-gnu`
  - `cargo check --examples --target=aarch64-apple-darwin`
- Example programs in `examples/` are the primary manual smoke tests; `examples/run_all.sh` runs them sequentially.
- For Windows behavior from Linux, README documents using Wine after `cargo build --examples --target=x86_64-pc-windows-gnu`.

## Practical Guardrails for Edits
- Keep public API minimal and data-oriented (plain structs + enums, no builder pattern currently).
- Match existing formatting/style (tabs, concise helper functions, `Option`-based cancel semantics).
- Avoid introducing extra dependencies unless required by platform API usage.
- Update/add an example in `examples/` when adding a new dialog capability.
