use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIconBuilder};
use wry::WebViewBuilder;

use crate::{config, html, privilege, purge, settings_html, stats, toast};

#[derive(Debug)]
enum UserEvent {
    MenuClicked(MenuId),
    RefreshStats,
    PurgeComplete(String),
    SettingsSaved(config::Settings),
    SettingsCancelled,
}

pub fn run() {
    // Check elevation
    if !privilege::is_elevated() {
        eprintln!("Warning: Running without Administrator privileges. Purge operations will fail.");
        eprintln!("Consider restarting as Administrator for full functionality.\n");
    } else if let Err(e) = privilege::elevate_for_purge() {
        eprintln!("Warning: Could not enable purge privileges: {e}\n");
    }

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // --- Build tray menu ---
    let stats_item = MenuItem::new("Stats", true, None);
    let settings_item = MenuItem::new("Settings", true, None);
    let purge_sub = Submenu::new("Purge Now", true);
    let ws_item = MenuItem::new("Working Sets", true, None);
    let sb_item = MenuItem::new("Standby List", true, None);
    let sbl_item = MenuItem::new("Standby (Low Priority)", true, None);
    let mod_item = MenuItem::new("Modified List", true, None);
    let all_item = MenuItem::new("All", true, None);
    let exit_item = MenuItem::new("Exit", true, None);

    let _ = purge_sub.append_items(&[
        &ws_item,
        &sb_item,
        &sbl_item,
        &mod_item,
        &PredefinedMenuItem::separator(),
        &all_item,
    ]);

    let menu = Menu::new();
    let _ = menu.append_items(&[
        &stats_item,
        &settings_item,
        &PredefinedMenuItem::separator(),
        &purge_sub,
        &PredefinedMenuItem::separator(),
        &exit_item,
    ]);

    // Capture menu item IDs for matching
    let id_stats = stats_item.id().clone();
    let id_settings = settings_item.id().clone();
    let id_ws = ws_item.id().clone();
    let id_sb = sb_item.id().clone();
    let id_sbl = sbl_item.id().clone();
    let id_mod = mod_item.id().clone();
    let id_all = all_item.id().clone();
    let id_exit = exit_item.id().clone();

    // --- Create tray icon ---
    let icon = create_tray_icon();
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Memory Pressure Agent")
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon");

    // Forward menu events to the event loop with the actual menu ID
    let menu_proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let _ = menu_proxy.send_event(UserEvent::MenuClicked(event.id));
    }));

    // --- State ---
    let mut window: Option<tao::window::Window> = None;
    let mut webview: Option<wry::WebView> = None;
    let mut settings_window: Option<tao::window::Window> = None;
    let mut settings_webview: Option<wry::WebView> = None;
    let settings = Arc::new(Mutex::new(config::Settings::load()));

    // --- Spawn monitoring thread ---
    spawn_monitor_thread(Arc::clone(&settings));

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::MenuClicked(ref eid)) => {
                if *eid == id_stats {
                    if window.is_none() {
                        let ipc_proxy = proxy.clone();
                        let (w, wv) = create_stats_window(event_loop, ipc_proxy);
                        window = Some(w);
                        webview = Some(wv);
                        refresh_stats_webview(webview.as_ref().unwrap());
                    } else {
                        if let Some(w) = &window {
                            w.set_visible(true);
                            w.set_focus();
                        }
                        refresh_stats_webview(webview.as_ref().unwrap());
                    }
                } else if *eid == id_settings {
                    if settings_window.is_none() {
                        let ipc_proxy = proxy.clone();
                        let (w, wv) = create_settings_window(event_loop, ipc_proxy);
                        settings_window = Some(w);
                        settings_webview = Some(wv);
                        let s = settings.lock().unwrap();
                        inject_settings(settings_webview.as_ref().unwrap(), &s);
                    } else {
                        if let Some(w) = &settings_window {
                            w.set_visible(true);
                            w.set_focus();
                        }
                        let s = settings.lock().unwrap();
                        inject_settings(settings_webview.as_ref().unwrap(), &s);
                    }
                } else if *eid == id_ws {
                    spawn_purge(&proxy, "Working Sets", PurgeKind::WorkingSets);
                } else if *eid == id_sb {
                    spawn_purge(&proxy, "Standby List", PurgeKind::Standby);
                } else if *eid == id_sbl {
                    spawn_purge(&proxy, "Standby (Low)", PurgeKind::StandbyLow);
                } else if *eid == id_mod {
                    spawn_purge(&proxy, "Modified List", PurgeKind::Modified);
                } else if *eid == id_all {
                    spawn_purge(&proxy, "All", PurgeKind::All);
                } else if *eid == id_exit {
                    *control_flow = ControlFlow::ExitWithCode(0);
                }
            }
            Event::UserEvent(UserEvent::RefreshStats) => {
                if let Some(wv) = &webview {
                    refresh_stats_webview(wv);
                }
            }
            Event::UserEvent(UserEvent::PurgeComplete(msg)) => {
                eprintln!("{msg}"); // Also log to console
                if let Some(wv) = &webview {
                    refresh_stats_webview(wv);
                }
            }
            Event::UserEvent(UserEvent::SettingsSaved(new_settings)) => {
                {
                    let mut s = settings.lock().unwrap();
                    *s = new_settings;
                    if let Err(e) = s.save() {
                        eprintln!("Failed to save settings: {e}");
                    }
                }
                if let Some(wv) = &settings_webview {
                    let _ = wv.evaluate_script("showToast('Settings saved')");
                }
            }
            Event::UserEvent(UserEvent::SettingsCancelled) => {
                if let Some(w) = &settings_window {
                    w.set_visible(false);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ref window_id,
                ..
            } => {
                // Hide the window instead of destroying
                if let Some(w) = &window {
                    if w.id() == *window_id {
                        w.set_visible(false);
                    }
                }
                if let Some(w) = &settings_window {
                    if w.id() == *window_id {
                        w.set_visible(false);
                    }
                }
            }
            _ => {}
        }
    });
}

fn create_stats_window(
    event_loop: &tao::event_loop::EventLoopWindowTarget<UserEvent>,
    ipc_proxy: tao::event_loop::EventLoopProxy<UserEvent>,
) -> (tao::window::Window, wry::WebView) {
    let window = WindowBuilder::new()
        .with_title("MPA — Memory Statistics")
        .with_inner_size(tao::dpi::LogicalSize::new(720.0, 700.0))
        .with_window_icon(Some(create_window_icon()))
        .build(event_loop)
        .expect("Failed to create stats window");

    let webview = WebViewBuilder::new()
        .with_html(html::STATS_HTML)
        .with_ipc_handler(move |request| {
            let body = request.body();
            if body == "refresh" {
                let _ = ipc_proxy.send_event(UserEvent::RefreshStats);
            }
        })
        .build(&window)
        .expect("Failed to create WebView");

    (window, webview)
}

fn create_settings_window(
    event_loop: &tao::event_loop::EventLoopWindowTarget<UserEvent>,
    ipc_proxy: tao::event_loop::EventLoopProxy<UserEvent>,
) -> (tao::window::Window, wry::WebView) {
    let window = WindowBuilder::new()
        .with_title("MPA — Settings")
        .with_inner_size(tao::dpi::LogicalSize::new(620.0, 480.0))
        .with_window_icon(Some(create_window_icon()))
        .build(event_loop)
        .expect("Failed to create settings window");

    let webview = WebViewBuilder::new()
        .with_html(settings_html::SETTINGS_HTML)
        .with_ipc_handler(move |request| {
            let body = request.body();
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(body) {
                match msg.get("cmd").and_then(|c| c.as_str()) {
                    Some("save") => {
                        if let Some(s) = msg.get("settings") {
                            if let Ok(new_settings) =
                                serde_json::from_value::<config::Settings>(s.clone())
                            {
                                let _ =
                                    ipc_proxy.send_event(UserEvent::SettingsSaved(new_settings));
                            }
                        }
                    }
                    Some("cancel") => {
                        let _ = ipc_proxy.send_event(UserEvent::SettingsCancelled);
                    }
                    _ => {}
                }
            }
        })
        .build(&window)
        .expect("Failed to create settings WebView");

    (window, webview)
}

fn inject_settings(webview: &wry::WebView, settings: &config::Settings) {
    let json = serde_json::to_string(settings).unwrap_or_default();
    let script = format!("loadSettings({json})");
    let _ = webview.evaluate_script(&script);
}

/// Spawn a background thread that periodically checks memory stats against thresholds.
/// Uses hysteresis: an alert fires once when the threshold is crossed, and only re-fires
/// after the value recovers below the threshold and crosses again.
fn spawn_monitor_thread(settings: Arc<Mutex<config::Settings>>) {
    thread::spawn(move || {
        const POLL_INTERVAL: Duration = Duration::from_secs(5);

        // Track whether each area is currently in alert state (above threshold).
        let mut alerted_memory_load = false;
        let mut alerted_available = false;
        let mut alerted_modified = false;
        let mut alerted_standby = false;

        loop {
            thread::sleep(POLL_INTERVAL);

            let Ok(s) = stats::collect_stats() else {
                continue;
            };

            let cfg = settings.lock().unwrap().clone();

            // Memory load: alert when ABOVE threshold → purge all
            if let Some(action) = check_threshold_above(
                &cfg.memory_load,
                s.memory_load_percent as f64,
                &mut alerted_memory_load,
            ) {
                let current = format!("{}%", s.memory_load_percent);
                let threshold = format!("{}%", cfg.memory_load.warning);
                handle_threshold_action(action, "Memory Load", &current, &threshold, || {
                    let _ = purge::purge_working_sets();
                    let _ = purge::purge_modified();
                    let _ = purge::purge_standby(false);
                });
            }

            // Available memory: alert when BELOW threshold → purge all
            if let Some(action) = check_threshold_below(
                &cfg.available_memory,
                s.available_physical_mb,
                &mut alerted_available,
            ) {
                let current = format!("{:.0} MB", s.available_physical_mb);
                let threshold = format!("{:.0} MB", cfg.available_memory.warning);
                handle_threshold_action(action, "Available Memory", &current, &threshold, || {
                    let _ = purge::purge_working_sets();
                    let _ = purge::purge_modified();
                    let _ = purge::purge_standby(false);
                });
            }

            // Modified list: alert when ABOVE threshold → flush modified
            if let Some(action) = check_threshold_above(
                &cfg.modified_list,
                s.modified_mb,
                &mut alerted_modified,
            ) {
                let current = format!("{:.0} MB", s.modified_mb);
                let threshold = format!("{:.0} MB", cfg.modified_list.warning);
                handle_threshold_action(action, "Modified List", &current, &threshold, || {
                    let _ = purge::purge_modified();
                });
            }

            // Standby list: alert when ABOVE threshold → purge standby
            if let Some(action) = check_threshold_above(
                &cfg.standby_list,
                s.standby_mb,
                &mut alerted_standby,
            ) {
                let current = format!("{:.0} MB", s.standby_mb);
                let threshold = format!("{:.0} MB", cfg.standby_list.warning);
                handle_threshold_action(action, "Standby List", &current, &threshold, || {
                    let _ = purge::purge_standby(false);
                });
            }
        }
    });
}

/// Execute the configured action for a threshold breach.
fn handle_threshold_action(
    action: config::ThresholdAction,
    area: &str,
    current: &str,
    threshold: &str,
    purge_fn: impl FnOnce(),
) {
    match action {
        config::ThresholdAction::Notify => {
            toast::alert_pressure(area, current, threshold);
        }
        config::ThresholdAction::Purge => {
            toast::alert_pressure(area, current, &format!("{threshold} — purging"));
            purge_fn();
        }
        config::ThresholdAction::None => {}
    }
}

/// Returns `Some(action)` when value crosses above `cfg.warning` for the first time.
/// Resets when value drops back below.
fn check_threshold_above(
    cfg: &config::ThresholdConfig,
    value: f64,
    alerted: &mut bool,
) -> Option<config::ThresholdAction> {
    if cfg.action == config::ThresholdAction::None {
        *alerted = false;
        return None;
    }
    if value >= cfg.warning {
        if !*alerted {
            *alerted = true;
            return Some(cfg.action);
        }
    } else {
        *alerted = false;
    }
    None
}

/// Returns `Some(action)` when value drops below `cfg.warning` for the first time.
/// Resets when value rises back above.
fn check_threshold_below(
    cfg: &config::ThresholdConfig,
    value: f64,
    alerted: &mut bool,
) -> Option<config::ThresholdAction> {
    if cfg.action == config::ThresholdAction::None {
        *alerted = false;
        return None;
    }
    if value <= cfg.warning {
        if !*alerted {
            *alerted = true;
            return Some(cfg.action);
        }
    } else {
        *alerted = false;
    }
    None
}

fn refresh_stats_webview(webview: &wry::WebView) {
    match stats::collect_stats() {
        Ok(s) => {
            let json = serde_json::to_string(&s).unwrap_or_default();
            let script = format!("updateStats({json})");
            let _ = webview.evaluate_script(&script);
        }
        Err(e) => {
            let escaped = e.to_string().replace('\\', "\\\\").replace('\'', "\\'");
            let _ = webview.evaluate_script(&format!("showError('{escaped}')"));
        }
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

fn spawn_purge(
    proxy: &tao::event_loop::EventLoopProxy<UserEvent>,
    label: &str,
    kind: PurgeKind,
) {
    let label = label.to_string();
    let proxy = proxy.clone();

    thread::spawn(move || {
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
                ws.map(|r| {
                    format!(
                        "Trimmed {} processes + flushed + purged",
                        r.processes_trimmed
                    )
                })
            }
        };

        let after_avail = stats::collect_stats()
            .map(|s| s.available_physical_mb)
            .unwrap_or(0.0);
        let freed = after_avail - before_avail;

        let message = match result {
            Ok(detail) => format!("Purge {label}: {detail} (freed {freed:+.1} MB)"),
            Err(e) => format!("Purge {label} failed: {e}"),
        };

        let _ = proxy.send_event(UserEvent::PurgeComplete(message));
    });
}

/// Load the embedded app icon PNG and decode to RGBA at the given size.
fn load_icon_rgba(target_size: u32) -> (Vec<u8>, u32, u32) {
    let png_bytes = include_bytes!("../docs/appicon.png");
    let img = image::load_from_memory(png_bytes).expect("Failed to decode embedded icon PNG");
    let resized = img.resize_exact(
        target_size,
        target_size,
        image::imageops::FilterType::Lanczos3,
    );
    let rgba = resized.to_rgba8();
    let (w, h) = rgba.dimensions();
    (rgba.into_raw(), w, h)
}

fn create_tray_icon() -> Icon {
    let (rgba, w, h) = load_icon_rgba(32);
    Icon::from_rgba(rgba, w, h).expect("Failed to create tray icon")
}

fn create_window_icon() -> tao::window::Icon {
    let (rgba, w, h) = load_icon_rgba(256);
    tao::window::Icon::from_rgba(rgba, w, h).expect("Failed to create window icon")
}
