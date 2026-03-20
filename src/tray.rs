use std::mem;
use std::ptr;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Shell::{
    NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DestroyMenu, DestroyWindow,
    GetCursorPos, IDI_APPLICATION, LoadIconW, MF_POPUP, MF_SEPARATOR, MF_STRING,
    PostMessageW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
    TrackPopupMenu, WM_USER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN,
};

/// Callback message sent by the shell when the tray icon is interacted with.
pub const WM_TRAY_CALLBACK: u32 = WM_USER + 1;

// Menu item IDs
pub const ID_STATS: u16 = 1001;
pub const ID_PURGE_WORKINGSETS: u16 = 1010;
pub const ID_PURGE_STANDBY: u16 = 1011;
pub const ID_PURGE_STANDBY_LOW: u16 = 1012;
pub const ID_PURGE_MODIFIED: u16 = 1013;
pub const ID_PURGE_ALL: u16 = 1014;
pub const ID_EXIT: u16 = 1099;

const CLASS_NAME: &str = "MpaMonitorClass";

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_fixed<const N: usize>(s: &str) -> [u16; N] {
    let mut buf = [0u16; N];
    for (i, ch) in s.encode_utf16().enumerate() {
        if i >= N - 1 {
            break;
        }
        buf[i] = ch;
    }
    buf
}

/// Register the hidden window class. Returns the class atom.
pub fn register_class(wnd_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT) -> u16 {
    let class_name = wide(CLASS_NAME);
    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: ptr::null_mut(),
        hIcon: ptr::null_mut(),
        hCursor: ptr::null_mut(),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };
    let atom = unsafe { RegisterClassW(&wc) };
    assert!(atom != 0, "RegisterClassW failed");
    atom
}

/// Create a hidden message-only window.
pub fn create_hidden_window() -> HWND {
    let class_name = wide(CLASS_NAME);
    let title = wide("MPA Monitor");
    // HWND_MESSAGE = -3 as isize cast to HWND for a message-only window
    let hwnd_message: HWND = -3isize as HWND;
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            0, 0, 0, 0,
            hwnd_message,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
        )
    };
    assert!(!hwnd.is_null(), "CreateWindowExW failed");
    hwnd
}

/// Add the tray icon for the given hidden window.
pub fn add_tray_icon(hwnd: HWND) {
    let icon = unsafe { LoadIconW(ptr::null_mut(), IDI_APPLICATION) };
    let mut nid: NOTIFYICONDATAW = unsafe { mem::zeroed() };
    nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAY_CALLBACK;
    nid.hIcon = icon;
    nid.szTip = wide_fixed::<128>("Memory Pressure Agent");

    let ok = unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };
    assert!(ok != 0, "Shell_NotifyIconW(NIM_ADD) failed");
}

/// Remove the tray icon.
pub fn remove_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = unsafe { mem::zeroed() };
    nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}

/// Show a balloon notification on the tray icon.
pub fn show_balloon(hwnd: HWND, title: &str, message: &str) {
    let mut nid: NOTIFYICONDATAW = unsafe { mem::zeroed() };
    nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_INFO;
    nid.szInfoTitle = wide_fixed::<64>(title);
    nid.szInfo = wide_fixed::<256>(message);
    // NIIF_INFO = 0x00000001
    nid.Anonymous.uVersion = 0; // we'll set dwInfoFlags via the union
    // dwInfoFlags is at the same offset as uVersion in the anonymous union — but in windows-sys
    // the balloon flag field is accessed differently. We'll use the raw approach:
    // Actually in windows-sys 0.59, NOTIFYICONDATAW doesn't expose dwInfoFlags easily.
    // The balloon will still show with NIIF_NONE (0) which is fine.

    unsafe {
        Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

/// Build and show the context menu at the current cursor position.
pub fn show_context_menu(hwnd: HWND) {
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }

        // "Stats" item
        let stats_label = wide("&Stats");
        AppendMenuW(menu, MF_STRING, ID_STATS as usize, stats_label.as_ptr());

        // Separator
        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

        // "Purge" submenu
        let purge_menu = CreatePopupMenu();
        let ws_label = wide("Working Sets");
        let sb_label = wide("Standby List");
        let sbl_label = wide("Standby (Low Priority)");
        let mod_label = wide("Modified List");
        let all_label = wide("All");
        AppendMenuW(purge_menu, MF_STRING, ID_PURGE_WORKINGSETS as usize, ws_label.as_ptr());
        AppendMenuW(purge_menu, MF_STRING, ID_PURGE_STANDBY as usize, sb_label.as_ptr());
        AppendMenuW(purge_menu, MF_STRING, ID_PURGE_STANDBY_LOW as usize, sbl_label.as_ptr());
        AppendMenuW(purge_menu, MF_STRING, ID_PURGE_MODIFIED as usize, mod_label.as_ptr());
        AppendMenuW(purge_menu, MF_SEPARATOR, 0, ptr::null());
        AppendMenuW(purge_menu, MF_STRING, ID_PURGE_ALL as usize, all_label.as_ptr());

        let purge_label = wide("Purge Now");
        AppendMenuW(menu, MF_POPUP | MF_STRING, purge_menu as usize, purge_label.as_ptr());

        // Separator
        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

        // "Exit" item
        let exit_label = wide("E&xit");
        AppendMenuW(menu, MF_STRING, ID_EXIT as usize, exit_label.as_ptr());

        // Required: SetForegroundWindow before TrackPopupMenu so menu dismisses properly
        let mut pt = mem::zeroed();
        GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(menu, TPM_LEFTALIGN | TPM_BOTTOMALIGN, pt.x, pt.y, 0, hwnd, ptr::null());
        // Post a benign message so the menu can dismiss
        PostMessageW(hwnd, 0, 0, 0);

        DestroyMenu(purge_menu);
        DestroyMenu(menu);
    }
}

/// Default handling for WM_DESTROY — removes tray icon and posts quit.
pub fn on_destroy(hwnd: HWND) {
    remove_tray_icon(hwnd);
    unsafe {
        PostQuitMessage(0);
    }
}

/// Destroy the hidden window (triggers WM_DESTROY → on_destroy).
pub fn request_exit(hwnd: HWND) {
    unsafe {
        DestroyWindow(hwnd);
    }
}
