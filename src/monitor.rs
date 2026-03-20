use std::thread;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, DispatchMessageW, GetMessageW, MSG, TranslateMessage,
    WM_COMMAND, WM_DESTROY, WM_RBUTTONUP,
};

use crate::privilege;
use crate::purge;
use crate::stats;
use crate::tray;
use crate::statswindow;

/// Global hidden-window handle, set once during init.
static mut G_HWND: HWND = std::ptr::null_mut();
/// Global stats-window handle.
static mut G_STATS_HWND: HWND = std::ptr::null_mut();

/// Entry point for monitor (tray) mode.
pub fn run() {
    // Check elevation
    if !privilege::is_elevated() {
        eprintln!("Warning: Running without Administrator privileges. Purge operations will fail.");
        eprintln!("Consider restarting as Administrator for full functionality.");
    } else if let Err(e) = privilege::elevate_for_purge() {
        eprintln!("Warning: Could not enable purge privileges: {e}");
    }

    // Register window classes
    tray::register_class(wnd_proc);
    statswindow::register_class();

    // Create windows
    let hwnd = tray::create_hidden_window();
    let stats_hwnd = statswindow::create_window();

    unsafe {
        G_HWND = hwnd;
        G_STATS_HWND = stats_hwnd;
    }

    tray::add_tray_icon(hwnd);

    // Win32 message loop
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            tray::WM_TRAY_CALLBACK => {
                let event = (lparam & 0xFFFF) as u32;
                if event == WM_RBUTTONUP {
                    tray::show_context_menu(hwnd);
                }
                0
            }
            WM_COMMAND => {
                let id = (wparam & 0xFFFF) as u16;
                handle_menu_command(hwnd, id);
                0
            }
            WM_DESTROY => {
                tray::on_destroy(hwnd);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn handle_menu_command(hwnd: HWND, id: u16) {
    match id {
        tray::ID_STATS => {
            let stats_hwnd = unsafe { G_STATS_HWND };
            if !stats_hwnd.is_null() {
                statswindow::show(stats_hwnd);
            }
        }
        tray::ID_PURGE_WORKINGSETS => spawn_purge(hwnd, "Working Sets", PurgeKind::WorkingSets),
        tray::ID_PURGE_STANDBY => spawn_purge(hwnd, "Standby List", PurgeKind::Standby),
        tray::ID_PURGE_STANDBY_LOW => spawn_purge(hwnd, "Standby (Low)", PurgeKind::StandbyLow),
        tray::ID_PURGE_MODIFIED => spawn_purge(hwnd, "Modified List", PurgeKind::Modified),
        tray::ID_PURGE_ALL => spawn_purge(hwnd, "All", PurgeKind::All),
        tray::ID_EXIT => tray::request_exit(hwnd),
        _ => {}
    }
}

#[derive(Clone, Copy)]
enum PurgeKind {
    WorkingSets,
    Standby,
    StandbyLow,
    Modified,
    All,
}

fn spawn_purge(hwnd: HWND, label: &str, kind: PurgeKind) {
    let label = label.to_string();
    let hwnd_raw = hwnd as usize; // HWND is a pointer; cast to usize for Send

    thread::spawn(move || {
        let hwnd: HWND = hwnd_raw as HWND;

        // Collect before stats
        let before_avail = stats::collect_stats()
            .map(|s| s.available_physical_mb)
            .unwrap_or(0.0);

        let result = match kind {
            PurgeKind::WorkingSets => purge::purge_working_sets()
                .map(|r| format!("Trimmed {} processes", r.processes_trimmed)),
            PurgeKind::Standby => purge::purge_standby(false).map(|_| "Done".to_string()),
            PurgeKind::StandbyLow => purge::purge_standby(true).map(|_| "Done".to_string()),
            PurgeKind::Modified => purge::purge_modified().map(|_| "Done".to_string()),
            PurgeKind::All => {
                let ws = purge::purge_working_sets();
                let _ = purge::purge_modified();
                let _ = purge::purge_standby(false);
                ws.map(|r| format!("Trimmed {} processes + flushed + purged", r.processes_trimmed))
            }
        };

        let after_avail = stats::collect_stats()
            .map(|s| s.available_physical_mb)
            .unwrap_or(0.0);

        let freed = after_avail - before_avail;

        let message = match result {
            Ok(detail) => format!("Purge {label}: {detail}\nFreed {freed:+.1} MB"),
            Err(e) => format!("Purge {label} failed: {e}"),
        };

        tray::show_balloon(hwnd, "MPA Purge", &message);

        // Refresh the stats window if it's visible
        let stats_hwnd = unsafe { G_STATS_HWND };
        if !stats_hwnd.is_null() && statswindow::is_visible(stats_hwnd) {
            statswindow::post_refresh(stats_hwnd);
        }
    });
}
