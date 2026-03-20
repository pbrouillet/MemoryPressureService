use std::sync::OnceLock;

use winrt_toast_reborn::{Toast, ToastDuration, ToastManager};

const AUMID: &str = "PBrouillet.MemoryPressureAgent";
const DISPLAY_NAME: &str = "Memory Pressure Agent";

static REGISTERED: OnceLock<bool> = OnceLock::new();

/// Ensure the app is registered for toast notifications (writes to registry once).
fn ensure_registered() {
    REGISTERED.get_or_init(|| {
        let icon = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("appicon.ico")))
            .filter(|p| p.exists());
        let icon_ref = icon.as_deref();

        if let Err(e) = winrt_toast_reborn::register(AUMID, DISPLAY_NAME, icon_ref) {
            eprintln!("Warning: Could not register for toast notifications: {e}");
        }
        true
    });
}

/// Send a simple informational toast.
pub fn notify(title: &str, body: &str) {
    ensure_registered();
    let manager = ToastManager::new(AUMID);
    let mut toast = Toast::new();
    toast.text1(title).text2(body);
    if let Err(e) = manager.show(&toast) {
        eprintln!("Toast error: {e}");
    }
}

/// Send an urgent memory pressure alert for a specific area.
pub fn alert_pressure(area: &str, current: &str, threshold: &str) {
    ensure_registered();
    let manager = ToastManager::new(AUMID);
    let mut toast = Toast::new();
    toast
        .text1("⚠ Memory Pressure Warning")
        .text2(format!("{area}: {current} (threshold: {threshold})"))
        .duration(ToastDuration::Long);
    if let Err(e) = manager.show(&toast) {
        eprintln!("Toast error: {e}");
    }
}
