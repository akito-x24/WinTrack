use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_autostart::MacosLauncher;

mod browser_pwa;
mod database;
mod export;
mod monitoring;
mod services;
mod tray;

pub use database::Database;
pub use services::AppState;

// ─── Tauri Commands ───────────────────────────────────────────────────────────

#[tauri::command]
async fn get_today_stats(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_today_stats().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_daily_usage(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    date: String,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_daily_usage(&date).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_weekly_usage(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    start_date: String,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .get_weekly_usage(&start_date)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_monthly_usage(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    year: i32,
    month: u32,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .get_monthly_usage(year, month)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_app_list(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_all_apps().map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_app_category(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_id: i64,
    category: String,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .update_app_category(app_id, &category)
        .map_err(|e| e.to_string())
}

/// Rename an app's display name. Rejects empty strings.
#[tauri::command]
async fn update_app_display_name(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_id: i64,
    display_name: String,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .update_app_display_name(app_id, &display_name)
        .map_err(|e| e.to_string())
}

/// Set whether an app is ignored by the tracker.
#[tauri::command]
async fn set_app_ignored(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_id: i64,
    ignored: bool,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .set_app_ignored(app_id, ignored)
        .map_err(|e| e.to_string())
}

/// Updates -
#[tauri::command]
async fn update_app_daily_limit(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_id: i64,
    limit_minutes: Option<i64>,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .update_app_daily_limit(app_id, limit_minutes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_app_reminder_interval(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_id: i64,
    interval_minutes: i64,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .update_app_reminder_interval(app_id, interval_minutes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_app_soft_lock_enabled(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .set_app_soft_lock_enabled(app_id, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_hourly_heatmap(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    date: String,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .get_hourly_heatmap(&date)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_settings(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_settings(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    settings: serde_json::Value,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .db
        .update_settings(&settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn pause_tracking(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.paused = true;
    Ok(())
}

#[tauri::command]
async fn resume_tracking(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.paused = false;
    Ok(())
}

#[tauri::command]
async fn is_tracking_paused(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<bool, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.paused)
}

#[tauri::command]
async fn export_data(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    format: String,
    start_date: String,
    end_date: String,
    output_path: String,
) -> Result<String, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    export::export_data(&state.db, &format, &start_date, &end_date, &output_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_30_day_average(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<i64, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_30_day_average().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_timeline(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    date: String,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.get_timeline(&date).map_err(|e| e.to_string())
}

#[tauri::command]
async fn close_process(process_name: String) -> Result<(), String> {
    std::process::Command::new("taskkill")
        .args(["/F", "/T", "/IM", &process_name])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn grant_app_more_time(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_id: i64,
) -> Result<(), String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.retain_soft_lock_day(&today);
    state.grant_soft_lock_extension(app_id, &today, 5 * 60);
    Ok(())
}

#[tauri::command]
async fn finish_soft_lock_warning(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_id: i64,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.clear_soft_lock_active(app_id);
    Ok(())
}

#[tauri::command]
async fn get_soft_lock_app_details(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    app_id: i64,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let (display_name, icon_data) = state
        .db
        .get_app_soft_lock_details(app_id)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "display_name": display_name,
        "icon_data": icon_data,
    }))
}

#[tauri::command]
async fn get_current_session(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "current_app":   state.current_app,
        "session_start": state.session_start,
        "is_idle":       state.is_idle,
    }))
}

#[tauri::command]
async fn reset_tracking_data(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.reset_tracking_data().map_err(|e| e.to_string())
}

#[tauri::command]
async fn factory_reset(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.db.factory_reset().map_err(|e| e.to_string())
}

// ─── App Entry ────────────────────────────────────────────────────────────────

pub fn run() {
    env_logger::init();

    let app_state = services::init_app_state().expect("Failed to initialize app state");
    let state_arc = Arc::new(Mutex::new(app_state));
    let state_for_monitor = Arc::clone(&state_arc);

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state_arc)
        .setup(|app| {
            let open_item = MenuItem::with_id(app, "open", "Open WinTrack", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let pause_item = MenuItem::with_id(app, "pause", "Pause Tracking", true, None::<&str>)?;
            let resume_item =
                MenuItem::with_id(app, "resume", "Resume Tracking", true, None::<&str>)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &open_item,
                    &sep1,
                    &pause_item,
                    &resume_item,
                    &sep2,
                    &quit_item,
                ],
            )?;

            TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("WinTrack")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| {
                    tray::handle_tray_event(app, event.id.as_ref());
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        tray::open_or_focus_window(tray.app_handle());
                    }
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let start_minimized = state_for_monitor
                    .lock()
                    .ok()
                    .and_then(|s| s.db.get_settings().ok())
                    .and_then(|v| v.get("start_minimized").and_then(|x| x.as_bool()))
                    .unwrap_or(false);

                if start_minimized {
                    let _ = window.hide();
                }
            }

            let handle = app.handle().clone();

            monitoring::start_monitoring_loop(state_for_monitor, handle);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" {
                    window.hide().unwrap();
                    api.prevent_close();
                } else if window.label() == "soft-lock" {
                    if let Some(state) = window.app_handle().try_state::<Arc<Mutex<AppState>>>() {
                        if let Ok(mut s) = state.lock() {
                            s.active_soft_lock_app_ids.clear();
                        }
                    }
                }
            }
            tauri::WindowEvent::Destroyed if window.label() == "soft-lock" => {
                if let Some(state) = window.app_handle().try_state::<Arc<Mutex<AppState>>>() {
                    if let Ok(mut s) = state.lock() {
                        s.active_soft_lock_app_ids.clear();
                    }
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_today_stats,
            get_daily_usage,
            get_weekly_usage,
            get_monthly_usage,
            get_app_list,
            get_30_day_average,
            update_app_category,
            update_app_display_name,
            set_app_ignored,
            update_app_daily_limit,
            update_app_reminder_interval,
            set_app_soft_lock_enabled,
            get_hourly_heatmap,
            get_settings,
            update_settings,
            pause_tracking,
            resume_tracking,
            is_tracking_paused,
            export_data,
            get_timeline,
            close_process,
            grant_app_more_time,
            finish_soft_lock_warning,
            get_soft_lock_app_details,
            get_current_session,
            reset_tracking_data,
            factory_reset,
        ])
        .build(tauri::generate_context!())
        .expect("Error building WinTrack app")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
