# Copilot Instructions for MemoryPressureAgent

## Build & Run

```sh
cargo build                    # Debug build
cargo build --release          # Release build
cargo run -- stats             # Run stats command
cargo run -- stats --json      # JSON output
cargo run -- purge workingsets # Requires admin terminal
```

No test suite exists yet. No linter configuration beyond default `cargo clippy`.

## Architecture

Windows-only Rust CLI (`mpa`) that reads Windows physical memory page list statistics via undocumented NT APIs and can purge memory regions (standby list, modified list, working sets) to relieve memory pressure.

**Data flow:** `main.rs` parses CLI → calls `stats::collect_stats()` or `purge::*` → renders via `display.rs`.

**Module responsibilities:**

- **`ntapi`** — Raw FFI layer to `ntdll.dll`. Dynamically loads `NtQuerySystemInformation` and `NtSetSystemInformation` via `GetProcAddress` at runtime (lazy `OnceLock`). Defines `SystemMemoryListInformation` (`#[repr(C)]`) matching the undocumented Windows struct and `MemoryListCommand` enum (values 0–5). This is the only module that touches `unsafe` transmute for function pointers.
- **`stats`** — Combines three data sources into `MemoryStats`: `GlobalMemoryStatusEx` (high-level), `NtQuerySystemInformation(80)` (page list breakdown), and `GetPerformanceInfo` (commit/cache/kernel pools). All page counts are converted to MB using the runtime page size.
- **`purge`** — Implements purge operations: `purge_working_sets` enumerates all PIDs via `EnumProcesses` + `EmptyWorkingSet`; `purge_standby`/`purge_modified` use `NtSetSystemInformation` memory list commands.
- **`privilege`** — Token manipulation: `is_elevated()` checks UAC elevation, `elevate_for_purge()` enables `SeProfileSingleProcessPrivilege` + `SeDebugPrivilege` via `AdjustTokenPrivileges`.
- **`display`** — Renders `MemoryStats` as `comfy-table` tables or `serde_json`. Also provides `print_diff()` for before/after comparisons.
- **`cli`** — Clap derive structs. Global `--json` flag, subcommands `stats` and `purge {workingsets|standby|modified|all}`.
- **`error`** — `MpaError` enum with `WinApi`, `Privilege`, and `General` variants. `MpaError::winapi()` auto-captures `GetLastError()`.

## Key Conventions

- **`windows-sys` not `windows` crate** — we use the raw FFI bindings crate (lighter, no COM). HANDLE is `*mut c_void`, not an integer; use `.is_null()` and `std::ptr::null_mut()`.
- **Undocumented NT APIs are loaded dynamically** — never link statically against `NtQuerySystemInformation`/`NtSetSystemInformation`. The `SystemMemoryListInformation` class ID is 80. The struct layout comes from Process Hacker headers / NtDoc and may change across Windows versions.
- **`comfy-table` is pinned to `=7.1.3`** — version 7.2+ uses let-chains which require a newer Rust compiler than our current toolchain (1.87).
- **Purge commands run work on a background thread** with an `indicatif` spinner on the main thread. The pattern uses `mpsc::channel` — a `PurgeTargetSimple` enum (non-clap, `Send`-safe) is sent to the worker since clap derives don't implement `Send`.
- **All purge commands show before/after stats with a delta line** — this is a core UX pattern; maintain it for any new purge operations.
- **Admin requirement** — purge commands check `is_elevated()` upfront and fail early with a user-friendly message. Stats commands work without elevation.
