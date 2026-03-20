# Copilot Instructions for MemoryPressureAgent

## Build & Run

```sh
cargo build                    # Debug build
cargo build --release          # Release build
cargo run -- stats             # Run stats command
cargo run -- stats --json      # JSON output
cargo run -- purge workingsets # Requires admin terminal
cargo run -- monitor           # Launch system-tray mode (admin recommended)
```

No test suite exists yet. No linter configuration beyond default `cargo clippy`.

## Architecture

Windows-only Rust CLI (`mpa`) that reads Windows physical memory page list statistics via undocumented NT APIs and can purge memory regions (standby list, modified list, working sets) to relieve memory pressure. Also supports a system-tray monitor mode with a stats window and quick purge menu.

**Data flow:** `main.rs` parses CLI → calls `stats::collect_stats()` or `purge::*` → renders via `display.rs`. In monitor mode: `main.rs` → `monitor::run()` → `tao` event loop with `tray-icon`/`muda` tray + `wry` WebView2 stats window.

**Module responsibilities:**

- **`ntapi`** — Raw FFI layer to `ntdll.dll`. Dynamically loads `NtQuerySystemInformation` and `NtSetSystemInformation` via `GetProcAddress` at runtime (lazy `OnceLock`). Defines `SystemMemoryListInformation` (`#[repr(C)]`) matching the undocumented Windows struct and `MemoryListCommand` enum (values 0–5). This is the only module that touches `unsafe` transmute for function pointers.
- **`stats`** — Combines three data sources into `MemoryStats`: `GlobalMemoryStatusEx` (high-level), `NtQuerySystemInformation(80)` (page list breakdown), and `GetPerformanceInfo` (commit/cache/kernel pools). All page counts are converted to MB using the runtime page size.
- **`purge`** — Implements purge operations: `purge_working_sets` enumerates all PIDs via `EnumProcesses` + `EmptyWorkingSet`; `purge_standby`/`purge_modified` use `NtSetSystemInformation` memory list commands.
- **`privilege`** — Token manipulation: `is_elevated()` checks UAC elevation, `elevate_for_purge()` enables `SeProfileSingleProcessPrivilege` + `SeDebugPrivilege` via `AdjustTokenPrivileges`.
- **`display`** — Renders `MemoryStats` as `comfy-table` tables or `serde_json`. Provides `render_table_to_string()` and `print_diff()` for before/after comparisons.
- **`cli`** — Clap derive structs. Global `--json` flag, subcommands `stats`, `purge {workingsets|standby|modified|all}`, and `monitor`.
- **`error`** — `MpaError` enum with `WinApi`, `Privilege`, and `General` variants. `MpaError::winapi()` auto-captures `GetLastError()`.
- **`monitor`** — Orchestrator for system-tray mode. Uses `tao` event loop, `tray-icon`+`muda` for the tray icon and context menu, and `wry` (WebView2) for the stats and settings windows. Purge operations spawn background threads and auto-refresh the stats webview on completion. Settings are loaded at startup and saved via IPC from the settings dialog.
- **`html`** — Embedded HTML/CSS/JS template for the stats WebView. Fluent Design styling (dark/light mode via `prefers-color-scheme`), card layout, color-coded page list bars, memory load gauge, Refresh button via IPC.
- **`settings_html`** — Embedded HTML/CSS/JS template for the settings WebView dialog. Matching Fluent Design form with per-area threshold inputs and action dropdowns. Communicates via IPC JSON messages (`save`/`cancel`).
- **`config`** — Application settings model. `Settings` and `ThresholdConfig` structs with serde Serialize/Deserialize, `load()`/`save()` to `mpa-settings.json` alongside the executable. Memory areas: memory_load (%), modified_list (MB), standby_list (MB), available_memory (MB). Actions: None, Notify, Purge.

## Key Conventions

- **`windows-sys` for core FFI, Tauri ecosystem for GUI** — `windows-sys` (raw FFI bindings) for ntdll/Win32 calls; `tao` (event loop), `wry` (WebView2), `tray-icon`+`muda` (tray/menus) for the monitor mode GUI. HANDLE is `*mut c_void`; use `.is_null()` and `std::ptr::null_mut()`.
- **Undocumented NT APIs are loaded dynamically** — never link statically against `NtQuerySystemInformation`/`NtSetSystemInformation`. The `SystemMemoryListInformation` class ID is 80. The struct layout comes from Process Hacker headers / NtDoc and may change across Windows versions.
- **`comfy-table` is pinned to `=7.1.3`** — version 7.2+ uses let-chains which require a newer Rust compiler than our current toolchain (1.87).
- **Purge commands run work on a background thread** with an `indicatif` spinner on the main thread. The pattern uses `mpsc::channel` — a `PurgeTargetSimple` enum (non-clap, `Send`-safe) is sent to the worker since clap derives don't implement `Send`.
- **All purge commands show before/after stats with a delta line** — this is a core UX pattern; maintain it for any new purge operations.
- **Admin requirement** — purge commands check `is_elevated()` upfront and fail early with a user-friendly message. Stats commands work without elevation.
