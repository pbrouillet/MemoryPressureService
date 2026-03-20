use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicIsize, Ordering};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontIndirectW, LOGFONTW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, IsWindowVisible,
    MoveWindow, PostMessageW, RegisterClassW, SendMessageW,
    SetWindowTextW, ShowWindow, WNDCLASSW,
    WM_CLOSE, WM_CREATE, WM_SETFONT, WM_SIZE, WM_USER,
    WS_CHILD, WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
    SW_HIDE, SW_SHOW, WM_COMMAND,
};

use crate::display;
use crate::stats;

const CLASS_NAME: &str = "MpaStatsWindowClass";
const EDIT_CLASS: &str = "EDIT";
const BUTTON_CLASS: &str = "BUTTON";

// Edit control styles (not all are in windows-sys)
const ES_MULTILINE: u32 = 0x0004;
const ES_READONLY: u32 = 0x0800;
const ES_AUTOVSCROLL: u32 = 0x0040;

const BS_PUSHBUTTON: u32 = 0x00000000;
const IDC_EDIT: u16 = 2001;
const IDC_REFRESH: u16 = 2002;

/// Custom message to trigger a stats refresh from any thread.
pub const WM_REFRESH_STATS: u32 = WM_USER + 100;

/// Height of the refresh button area at the bottom.
const BUTTON_HEIGHT: i32 = 36;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

static STATS_HWND: AtomicIsize = AtomicIsize::new(0);

/// Get the stats window HWND (if created).
#[allow(dead_code)]
pub fn hwnd() -> Option<HWND> {
    let val = STATS_HWND.load(Ordering::Relaxed);
    if val == 0 {
        None
    } else {
        Some(val as HWND)
    }
}

/// Register the stats window class.
pub fn register_class() -> u16 {
    let class_name = wide(CLASS_NAME);
    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(stats_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: ptr::null_mut(),
        hIcon: ptr::null_mut(),
        hCursor: ptr::null_mut(),
        hbrBackground: (5 + 1) as *mut _, // COLOR_WINDOW + 1
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };
    let atom = unsafe { RegisterClassW(&wc) };
    assert!(atom != 0, "RegisterClassW for stats window failed");
    atom
}

/// Create the stats window (initially hidden).
pub fn create_window() -> HWND {
    let class_name = wide(CLASS_NAME);
    let title = wide("MPA — Memory Statistics");
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            100, 100, 700, 600,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
        )
    };
    assert!(!hwnd.is_null(), "CreateWindowExW for stats window failed");
    STATS_HWND.store(hwnd as isize, Ordering::Relaxed);
    hwnd
}

/// Show the stats window (and refresh its content).
pub fn show(hwnd: HWND) {
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        // Force to foreground
        windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
    }
    refresh(hwnd);
}

/// Hide the stats window.
pub fn hide(hwnd: HWND) {
    unsafe {
        ShowWindow(hwnd, SW_HIDE);
    }
}

/// Check if the stats window is currently visible.
pub fn is_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd) != 0 }
}

/// Refresh the stats text.
pub fn refresh(hwnd: HWND) {
    let text = match stats::collect_stats() {
        Ok(s) => display::render_table_to_string(&s),
        Err(e) => format!("Error collecting stats: {e}"),
    };
    let wide_text = wide(&text);
    unsafe {
        let edit = windows_sys::Win32::UI::WindowsAndMessaging::GetDlgItem(hwnd, IDC_EDIT as i32);
        if !edit.is_null() {
            SetWindowTextW(edit, wide_text.as_ptr());
        }
    }
}

/// Send a WM_REFRESH_STATS message (can be called from any thread).
pub fn post_refresh(hwnd: HWND) {
    unsafe {
        PostMessageW(hwnd, WM_REFRESH_STATS, 0, 0);
    }
}

unsafe extern "system" fn stats_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let edit_class = wide(EDIT_CLASS);
                let edit = CreateWindowExW(
                    WS_EX_CLIENTEDGE,
                    edit_class.as_ptr(),
                    ptr::null(),
                    WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_TABSTOP
                        | ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL,
                    0, 0, 100, 100,
                    hwnd,
                    IDC_EDIT as isize as *mut _,
                    ptr::null_mut(),
                    ptr::null(),
                );

                // Set monospace font (Consolas)
                let font_name = wide("Consolas");
                let mut lf: LOGFONTW = mem::zeroed();
                lf.lfHeight = -14; // ~10pt
                lf.lfCharSet = 1; // DEFAULT_CHARSET
                for (i, ch) in font_name.iter().enumerate() {
                    if i >= 31 {
                        break;
                    }
                    lf.lfFaceName[i] = *ch;
                }
                let font = CreateFontIndirectW(&lf);
                if !font.is_null() {
                    SendMessageW(edit, WM_SETFONT, font as WPARAM, 1);
                }

                // Create Refresh button
                let btn_class = wide(BUTTON_CLASS);
                let btn_label = wide("Refresh");
                CreateWindowExW(
                    0,
                    btn_class.as_ptr(),
                    btn_label.as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
                    0, 0, 100, 30,
                    hwnd,
                    IDC_REFRESH as isize as *mut _,
                    ptr::null_mut(),
                    ptr::null(),
                );

                0
            }
            WM_SIZE => {
                let mut rc = mem::zeroed();
                GetClientRect(hwnd, &mut rc);
                let w = rc.right - rc.left;
                let h = rc.bottom - rc.top;

                let edit = windows_sys::Win32::UI::WindowsAndMessaging::GetDlgItem(hwnd, IDC_EDIT as i32);
                if !edit.is_null() {
                    MoveWindow(edit, 0, 0, w, h - BUTTON_HEIGHT, 1);
                }
                let btn = windows_sys::Win32::UI::WindowsAndMessaging::GetDlgItem(hwnd, IDC_REFRESH as i32);
                if !btn.is_null() {
                    let btn_w = 100;
                    MoveWindow(btn, (w - btn_w) / 2, h - BUTTON_HEIGHT + 3, btn_w, 30, 1);
                }
                0
            }
            WM_COMMAND => {
                let id = (wparam & 0xFFFF) as u16;
                if id == IDC_REFRESH {
                    refresh(hwnd);
                }
                0
            }
            WM_REFRESH_STATS => {
                refresh(hwnd);
                0
            }
            WM_CLOSE => {
                hide(hwnd);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
