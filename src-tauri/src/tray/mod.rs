use tauri::{AppHandle, Manager};

pub fn handle_tray_event(app: &AppHandle, id: &str) {
    match id {
        "open" => open_or_focus_window(app),
        "pause" => {
            if let Some(state) = app.try_state::<std::sync::Arc<std::sync::Mutex<crate::services::AppState>>>() {
                if let Ok(mut s) = state.lock() {
                    s.paused = true;
                    log::info!("Tracking paused from tray");
                }
            }
        }
        "resume" => {
            if let Some(state) = app.try_state::<std::sync::Arc<std::sync::Mutex<crate::services::AppState>>>() {
                if let Ok(mut s) = state.lock() {
                    s.paused = false;
                    log::info!("Tracking resumed from tray");
                }
            }
        }
        "quit" => {
            log::info!("Quitting FocusPulse");
            std::process::exit(0);
        }
        _ => {}
    }
}

pub fn open_or_focus_window(app: &AppHandle) {
    // Tauri v2: use get_webview_window
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
    }
}
