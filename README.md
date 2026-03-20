# mpa — Memory Pressure Agent

A Windows CLI tool that displays detailed physical memory statistics and can purge memory regions to relieve pressure. Includes a **system-tray monitor mode** for always-on memory management. Think of it as a scriptable, command-line [RAMMap](https://learn.microsoft.com/en-us/sysinternals/downloads/rammap).

## Features

- **Full page list breakdown** — zeroed, free, modified, and standby pages (by priority 0–7), sourced from the same undocumented NT APIs that RAMMap uses
- **System overview** — memory load, commit charge, system cache, kernel pools, process/thread/handle counts
- **Purge commands** — empty working sets, flush modified pages, purge standby list (all or low-priority only)
- **Before/after stats** — every purge shows a delta summary so you see exactly what changed
- **Table or JSON output** — `--json` flag for scripting and automation
- **System-tray monitor** — resident tray icon with context menu for quick purges, a stats window, and settings dialog
- **Persistent settings** — per-area warning thresholds and actions stored in `mpa-settings.json` alongside the executable

## Usage

```
mpa stats                        # Display memory statistics as a table
mpa stats --json                 # Output as JSON
mpa purge workingsets            # Empty all processes' working sets
mpa purge standby                # Purge the entire standby list
mpa purge standby --low-only     # Purge only low-priority standby pages
mpa purge modified               # Flush modified page list to disk
mpa purge all                    # All of the above in sequence
mpa monitor                     # Launch system-tray monitor mode
```

`stats` works as a regular user. All `purge` commands and `monitor` mode require an **Administrator** terminal.

### Monitor mode

`mpa monitor` starts a resident system-tray application with:
- **Tray icon** — right-click for context menu
- **Stats** — opens a modern **WebView2 window** with Fluent Design styling (dark/light mode, card layout, color-coded bars, memory load gauge)
- **Settings** — opens a configuration dialog for warning thresholds and actions per memory area (memory load, available memory, modified list, standby list). Settings are saved to `mpa-settings.json` next to the executable.
- **Purge Now** — submenu with Working Sets, Standby List, Standby (Low Priority), Modified List, and All
- **Exit** — removes the tray icon and quits

Purge operations run on a background thread; the stats window auto-refreshes on completion. The stats window uses WebView2 (Edge-based) for a native Windows 11 look and feel.

### Example output

```
┌───────────────────────────────┬───────────────────────┐
│ System Overview               ┆ Value                 │
╞═══════════════════════════════╪═══════════════════════╡
│ Memory Load                   ┆ 44%                   │
│ Total Physical                ┆ 97964.7 MB            │
│ Available Physical            ┆ 54260.9 MB            │
│ Commit (Used / Limit)         ┆ 66465.0 / 102060.7 MB │
│ System Cache                  ┆ 49150.7 MB            │
│ Kernel Paged / Nonpaged       ┆ 2283.6 / 2762.6 MB    │
│ Processes / Threads / Handles ┆ 480 / 11212 / 287433  │
└───────────────────────────────┴───────────────────────┘

┌─────────────────┬────────────┬───────────┐
│ Page List       ┆ Pages      ┆ Size (MB) │
╞═════════════════╪════════════╪═══════════╡
│ Zeroed          ┆ 452,697    ┆ 1768.3    │
│ Free            ┆ 10,156     ┆ 39.7      │
│ Modified        ┆ 69,480     ┆ 271.4     │
│ Standby (total) ┆ 13,427,949 ┆ 52452.9   │
└─────────────────┴────────────┴───────────┘
```

After a purge:

```
✓ Trimmed 460 processes (17 skipped)

  ─── Delta ───
  Available:  +4470.5 MB  (54916.1 → 59386.6)
  Standby:    +3666.9 MB  (52496.7 → 56163.6)
  Modified:   +16087.4 MB  (254.6 → 16342.0)
```

## Building

Requires Rust 1.87+ and a Windows target.

```sh
cargo build --release
```

The binary is at `target/release/mpa.exe`.

## How it works

| Layer | API | Purpose |
|-------|-----|---------|
| Stats | `GlobalMemoryStatusEx` | Memory load, total/available RAM, commit |
| Stats | `GetPerformanceInfo` | Commit, cache, kernel pools, process counts |
| Stats | `NtQuerySystemInformation(80)` | Page list breakdown (zeroed, free, modified, standby×8) |
| Purge | `EmptyWorkingSet` + `EnumProcesses` | Trim each process's working set |
| Purge | `NtSetSystemInformation(80)` | Flush modified list, purge standby list |
| Privilege | `AdjustTokenPrivileges` | Enable `SeProfileSingleProcessPrivilege` + `SeDebugPrivilege` |

The `NtQuerySystemInformation` / `NtSetSystemInformation` calls with `SystemMemoryListInformation` (class 80) are **undocumented** — the same APIs that Sysinternals RAMMap uses. They are loaded dynamically from `ntdll.dll` at runtime.

## License

MIT
