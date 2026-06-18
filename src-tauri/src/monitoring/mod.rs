use crate::services::AppState;
use chrono::Local;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "windows")]
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::Foundation::{CloseHandle, PROPERTYKEY},
    Win32::System::Com::StructuredStorage::PropVariantToString,
    Win32::System::SystemInformation::GetTickCount,
    Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
    Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
    Win32::UI::Shell::PropertiesSystem::{
        IPropertyStore, PSGetPropertyKeyFromName, SHGetPropertyStoreForWindow,
    },
    Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    },
};

#[derive(Debug, Clone)]
pub struct ForegroundApp {
    pub app_name: String,
    pub executable_path: String,
    pub window_title: String,
}

#[cfg(target_os = "windows")]
pub fn get_foreground_app() -> Option<ForegroundApp> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        let window_title = if title_len > 0 {
            String::from_utf16_lossy(&title_buf[..title_len as usize])
        } else {
            String::new()
        };

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut path_buf = [0u16; 1024];
        let mut path_len = path_buf.len() as u32;
        let success = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(path_buf.as_mut_ptr()),
            &mut path_len,
        );
        let _ = CloseHandle(process);

        if success.is_err() || path_len == 0 {
            return None;
        }

        let executable_path = String::from_utf16_lossy(&path_buf[..path_len as usize]);
        let exe_name = std::path::Path::new(&executable_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| executable_path.clone());

        let (app_name, executable_path) = if is_uwp_host_process(&exe_name) {
            resolve_uwp_foreground_app(hwnd, &window_title)
                .unwrap_or((exe_name.clone(), executable_path.clone()))
        } else {
            (exe_name.clone(), executable_path.clone())
        };

        Some(ForegroundApp {
            app_name,
            executable_path,
            window_title,
        })
    }
}

#[cfg(target_os = "windows")]
fn is_uwp_host_process(exe_name: &str) -> bool {
    matches!(
        exe_name.to_lowercase().as_str(),
        "applicationframehost.exe" | "runtimebroker.exe" | "wwahost.exe"
    )
}

#[cfg(target_os = "windows")]
fn resolve_uwp_foreground_app(hwnd: windows::Win32::Foundation::HWND, window_title: &str) -> Option<(String, String)> {
    // UWP and packaged apps are often hosted by a generic Win32 process such as
    // ApplicationFrameHost.exe or RuntimeBroker.exe. The window itself carries
    // a package identity via Shell properties, which allows us to map the
    // foreground window back to the real app.
    unsafe {
        let property_store: IPropertyStore = SHGetPropertyStoreForWindow(hwnd).ok()?;
        let mut app_user_model_key: PROPERTYKEY = std::mem::zeroed();

        let app_user_model_id_name: Vec<u16> = "System.AppUserModel.ID"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        PSGetPropertyKeyFromName(PCWSTR(app_user_model_id_name.as_ptr()), &mut app_user_model_key)
            .ok()?;

        let propvar = property_store.GetValue(&app_user_model_key).ok()?;
        let mut buffer = [0u16; 512];
        PropVariantToString(&propvar, &mut buffer).ok()?;
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        let app_user_model_id = String::from_utf16_lossy(&buffer[..len]);
        if app_user_model_id.is_empty() {
            return None;
        }

        let app_name = if !window_title.trim().is_empty() {
            window_title.to_string()
        } else {
            app_user_model_id.clone()
        };

        Some((app_name, app_user_model_id))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_foreground_app() -> Option<ForegroundApp> {
    Some(ForegroundApp {
        app_name: "wintrack-dev".to_string(),
        executable_path: "/usr/bin/wintrack-dev".to_string(),
        window_title: "WinTrack Development".to_string(),
    })
}

#[cfg(target_os = "windows")]
pub fn get_idle_seconds() -> u64 {
    unsafe {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut info).as_bool() {
            let tick_count = GetTickCount();
            let idle_ms = tick_count.saturating_sub(info.dwTime);
            (idle_ms as u64) / 1000
        } else {
            0
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_idle_seconds() -> u64 {
    0
}

// ─── WinTrack own exe name (lowercased) - always ignored ───────────────────
const SELF_EXE: &str = "wintrack.exe";

pub fn start_monitoring_loop(state: Arc<Mutex<AppState>>, handle: AppHandle) {
    thread::spawn(move || {
        let mut current_app: Option<ForegroundApp> = None;
        let mut session_start: Option<Instant> = None;
        let mut session_start_str: Option<String> = None;
        let mut consecutive_fails: u32 = 0;

        println!("WinTrack monitoring loop started");

        loop {
            let (poll_ms, idle_threshold_secs, is_paused) = {
                match state.lock() {
                    Ok(s) => (
                        // s.db.get_polling_interval() as u64,
                        s.db.get_polling_interval().max(1000) as u64,
                        s.db.get_idle_threshold() * 60,
                        s.paused,
                    ),
                    Err(_) => (1000, 300, false),
                }
            };

            thread::sleep(Duration::from_millis(poll_ms));

            if is_paused {
                if current_app.is_some() {
                    flush_session(
                        &state,
                        &handle,
                        &current_app,
                        &session_start,
                        &session_start_str,
                        false,
                    );
                    current_app = None;
                    session_start = None;
                    session_start_str = None;
                }
                continue;
            }

            let idle_secs = get_idle_seconds();

            let is_idle = if idle_threshold_secs == 0 {
                false
            } else {
                idle_secs >= idle_threshold_secs as u64
            };
            let new_app = get_foreground_app();

            // Skip WinTrack itself - never track our own process
            let new_app = new_app.filter(|a| a.app_name.to_lowercase() != SELF_EXE);

            // If new app is ignored in DB, treat as no foreground app
            let new_app = new_app.and_then(|a| {
                let ignored = state
                    .lock()
                    .map(|s| s.db.is_app_ignored(&a.executable_path))
                    .unwrap_or(false);
                if ignored {
                    None
                } else {
                    Some(a)
                }
            });

            let should_flush = match (&current_app, &new_app) {
                (Some(cur), Some(new)) => cur.executable_path != new.executable_path || is_idle,
                (Some(_), None) => true,
                _ => false,
            };

            if should_flush {
                flush_session(
                    &state,
                    &handle,
                    &current_app,
                    &session_start,
                    &session_start_str,
                    false,
                );
                current_app = None;
                session_start = None;
                session_start_str = None;
            }

            if !is_idle {
                if let Some(ref app) = new_app {
                    if current_app.is_none() {
                        session_start = Some(Instant::now());
                        session_start_str =
                            Some(Local::now().format("%Y-%m-%dT%H:%M:%S").to_string());
                        current_app = Some(app.clone());
                    }
                }
            }

            if !is_idle {
                if let (Some(app), Some(start)) = (&current_app, &session_start) {
                    if let Ok(s) = state.lock() {
                        if let Ok((app_id, is_ignored)) =
                            s.db.upsert_app(&app.app_name, &app.executable_path)
                        {
                            if !is_ignored {
                                let current_session_seconds = start.elapsed().as_secs() as i64;

                                if let Ok(Some((today_usage, daily_limit))) =
                                    s.db.get_app_limit_status(app_id)
                                {
                                    let total_usage = today_usage + current_session_seconds;

                                    let limit_seconds = daily_limit * 60;
                                    if total_usage >= limit_seconds {
                                        let display_name =
                                            s.db.get_app_display_name(app_id)
                                                .unwrap_or_else(|_| app.app_name.clone());

                                        let today =
                                            chrono::Local::now().format("%Y-%m-%d").to_string();

                                        if s.db
                                            .should_send_limit_notification(app_id, &today)
                                            .unwrap_or(false)
                                        {
                                            let _ = handle
                                                .notification()
                                                .builder()
                                                .title("Daily Limit Reached")
                                                .body(&format!(
                                                    "{} has reached its daily limit.",
                                                    display_name
                                                ))
                                                .show();

                                            let _ =
                                                s.db.mark_limit_notification_sent(app_id, &today);

                                            let _ = s.db.mark_reminder_sent(app_id, total_usage);

                                            println!(
                                                "REAL-TIME LIMIT NOTIFICATION: {}",
                                                display_name
                                            );
                                        }

                                        let reminder_interval =
                                            s.db.get_app_reminder_interval(app_id).unwrap_or(0);

                                        if reminder_interval > 0
                                            && s.db
                                                .should_send_reminder(
                                                    app_id,
                                                    total_usage,
                                                    reminder_interval,
                                                )
                                                .unwrap_or(false)
                                        {
                                            let _ = handle

                                                    .notification()

                                                    .builder()

                                                    .title("Reminder")

                                                    .body(&format!(

                                                        "You're still using {} after exceeding its limit.",

                                                            display_name

                                                        ))

                                                    .show();

                                            let _ = s.db.mark_reminder_sent(app_id, total_usage);

                                            println!("REAL-TIME REMINDER: {}", display_name);

                                            if s.db.is_soft_lock_enabled(app_id).unwrap_or(false) {
                                                let _ = s.db.increment_soft_lock_counter(app_id);

                                                if let Ok(count) =
                                                    s.db.get_soft_lock_counter(app_id)
                                                {
                                                    println!(
                                                        "SOFT LOCK COUNT: {} -> {}",
                                                        display_name, count
                                                    );

                                                    const SOFT_LOCK_TRIGGER_REMINDERS: i64 = 1;

                                                    if count >= SOFT_LOCK_TRIGGER_REMINDERS {
                                                        println!(
                                                            "SOFT LOCK TRIGGERED: {}",
                                                            display_name
                                                        );

                                                        if handle
                                                            .get_webview_window("soft-lock")
                                                            .is_none()
                                                        {
                                                            let url = format!(
                                                                "/?softlock=1&app={}&process={}",
                                                                urlencoding::encode(&display_name),
                                                                urlencoding::encode(&app.app_name)
                                                            );

                                                            let _ = WebviewWindowBuilder::new(
                                                                &handle,
                                                                "soft-lock",
                                                                tauri::WebviewUrl::App(url.into()),
                                                            )
                                                            .inner_size(1280.0, 720.0)
                                                            .resizable(true)
                                                            .center()
                                                            .build();
                                                        }

                                                        let _ =
                                                            s.db.reset_soft_lock_counter(app_id);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            match state.lock() {
                Ok(mut s) => {
                    s.current_app = current_app.as_ref().map(|a| a.app_name.clone());
                    s.session_start = session_start_str.clone();
                    s.is_idle = is_idle;
                    consecutive_fails = 0;
                }
                Err(_) => {
                    consecutive_fails += 1;
                    if consecutive_fails > 10 {
                        log::error!("Failed to lock state repeatedly, sleeping...");
                        thread::sleep(Duration::from_secs(5));
                        consecutive_fails = 0;
                    }
                }
            }
        }
    });
}

fn flush_session(
    state: &Arc<Mutex<AppState>>,
    _handle: &AppHandle,
    current_app: &Option<ForegroundApp>,
    session_start: &Option<Instant>,
    session_start_str: &Option<String>,
    was_idle: bool,
) {
    if let (Some(app), Some(start), Some(start_str)) =
        (current_app, session_start, session_start_str)
    {
        let duration = start.elapsed().as_secs() as i64;
        if duration < 1 {
            return;
        }

        let end_str = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        if let Ok(s) = state.lock() {
            match s.db.upsert_app(&app.app_name, &app.executable_path) {
                Ok((app_id, is_ignored)) => {
                    if !is_ignored {
                        let _ = s.db.insert_session(
                            app_id,
                            &app.window_title,
                            start_str,
                            &end_str,
                            duration,
                            was_idle,
                        );
                        log::debug!(
                            "Flushed: {} ({}s idle={})",
                            app.app_name,
                            duration,
                            was_idle
                        );
                    }
                }
                Err(e) => log::error!("Failed to upsert app: {}", e),
            }
        }
    }
}
