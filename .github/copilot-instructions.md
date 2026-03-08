# Copilot Instructions for rustydialogs

## Big picture
- `src/lib.rs` is the API surface: plain data structs + enums (`MessageBox`, `FileDialog`, `TextInput`, `ColorPicker`, `Notification`) with thin forwarding methods.
- Platform dispatch is compile-time + runtime:
  - Linux/BSD: `src/linux/mod.rs` chooses backend once via `LazyLock<Backend>`.
  - Windows: `src/win32/mod.rs` chooses notification backend on Cargo features.
  - macOS: `src/macos/mod.rs` re-exports either `appkit` or `osascript` implementation.
- Design intent is minimal abstraction: backend functions return `Option<T>` where cancel/unsupported generally maps to `None`.

## Architecture pattern to keep
- For any dialog API change, update all three layers:
  1. Public type/method in `src/lib.rs`
  2. Per-platform dispatcher (`src/linux/mod.rs`, `src/win32/mod.rs`, `src/macos/mod.rs`)
  3. Concrete backend modules (e.g. `src/linux/zenity.rs`, `src/win32/file.rs`, `src/macos/osascript.rs`)
- Keep backend function names aligned (`message_box`, `pick_file`, `pick_files`, `save_file`, `choose_folder`, `choose_folders`, `text_input`, `color_picker`, `notify`).
- Preserve current style: tab indentation, concise helpers, no builder API.

## Backend-specific conventions
- Linux command backends (`kdialog`, `zenity`): build args as `&OsStr` using `os(...)`; invoke via shared helpers in `src/linux/mod.rs` (`invoke*`), not ad-hoc `Command` code.
- Linux path parsing: read stdout as bytes (`invoke_output_bytes` + `OsStrExt::from_bytes`) for filesystem results.
- Linux backend selection: honor `RUSTY_DIALOGS_BACKEND=gtk4|gtk3|xdg-portal|zenity|kdialog`; if unset, GTK features are preferred when compiled in.
- `src/linux/xdg_portal.rs` is intentionally partial: message box/text input/color picker return `None`; file/folder are implemented.
- Windows owner handling is active (`hwnd(p.owner)` in `src/win32/*`); Linux/macOS ignore `owner`.
- macOS default path uses AppleScript in `src/macos/osascript.rs`; keep best-effort semantics (timeouts/notification behavior vary by OS).

## Workflows that matter here
- Run checks for the backend and target you changed. Do not stop after `cargo check --examples` on host target only.
- Windows host (Windows checks only):
  - Default Windows backend changes (`src/win32/**`): `cargo check --examples`
  - WinRT toast changes (`src/win32/toast.rs` or toast wiring): `cargo check --examples --features winrt-toast`
- macOS host (macOS checks only):
  - AppleScript backend changes (`src/macos/osascript.rs` or osascript wiring): `cargo check --examples`
  - AppKit backend changes (`src/macos/appkit.rs` or appkit wiring): `cargo check --examples --features appkit`
- Linux host (full matrix; use this host to validate every platform/backend path):
  - Linux default backends (`src/linux/kdialog.rs`, `src/linux/zenity.rs`, `src/linux/mod.rs`): `cargo check --examples`
  - Linux GTK3 backend (`src/linux/gtk3/**`): `cargo check --examples --features gtk3`
  - Linux GTK4 backend (`src/linux/gtk4/**`): `cargo check --examples --features gtk4`
  - Linux XDG portal backend (`src/linux/xdg_portal.rs`): `cargo check --examples --features xdg-portal`
  - Linux notification code using libnotify (`src/linux/notify.rs` or GTK notification plumbing): `cargo check --examples --features libnotify`
  - Windows backend code from Linux (`src/win32/**`): `cargo check --examples --target=x86_64-pc-windows-gnu`
  - macOS backend code from Linux (`src/macos/**`): `cargo check --examples --target=aarch64-apple-darwin`
- Interactive smoke tests live in `examples/tests.rs`.
  - Run all: `cargo run --example tests`
  - Run one group: `cargo run --example tests -- m|o|s|f|t|c|n`
  - Backend matrix examples: see `readme.md` and `testreport.md`.
- Finish-up step for backend work and PR-ready changes:
  - Strongly recommend running the appropriate interactive test group for each backend touched via `cargo run --example tests`.
  - If asked, run the tests and capture stdout/stderr from that run.
  - Compare observed behavior against the corresponding backend notes in `testreport/` and `testreport.md`.
  - Discuss differences with the user and cross-check against the user’s notes before concluding behavior is expected.
  - Do not assume backend quirks are acceptable without explicit user verification.

## Practical guardrails
- Keep dependencies minimal; if new Win32 APIs are added, update `Cargo.toml` `windows.features`.
- When adding dialog capabilities, also update at least one relevant example in `examples/`.
- Prefer root-cause backend fixes over API-level workarounds; documented backend quirks are tracked in `testreport.md`.
