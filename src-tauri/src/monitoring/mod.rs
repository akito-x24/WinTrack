use crate::services::AppState;
use chrono::Local;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;

#[cfg(target_os = "windows")]
use windows::{
    core::PWSTR,
    Win32::Foundation::CloseHandle,
    Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
    Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
    Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    },
    Win32::System::SystemInformation::GetTickCount,
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
        let app_name = std::path::Path::new(&executable_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| executable_path.clone());

        Some(ForegroundApp { app_name, executable_path, window_title })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_foreground_app() -> Option<ForegroundApp> {
    Some(ForegroundApp {
        app_name: "focuspulse-dev".to_string(),
        executable_path: "/usr/bin/focuspulse-dev".to_string(),
        window_title: "FocusPulse Development".to_string(),
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
pub fn get_idle_seconds() -> u64 { 0 }

// ─── Focuspulse own exe name (lowercased) — always ignored ───────────────────
const SELF_EXE: &str = "focuspulse.exe";

pub fn start_monitoring_loop(state: Arc<Mutex<AppState>>, _handle: AppHandle) {
    thread::spawn(move || {
        let mut current_app: Option<ForegroundApp> = None;
        let mut session_start: Option<Instant> = None;
        let mut session_start_str: Option<String> = None;
        let mut consecutive_fails: u32 = 0;

        log::info!("FocusPulse monitoring loop started");

        loop {
            let (poll_ms, idle_threshold_secs, is_paused) = {
                match state.lock() {
                    Ok(s) => (s.db.get_polling_interval() as u64,
                              s.db.get_idle_threshold() * 60,
                              s.paused),
                    Err(_) => (1000, 300, false),
                }
            };

            thread::sleep(Duration::from_millis(poll_ms));

            if is_paused {
                if current_app.is_some() {
                    flush_session(&state, &current_app, &session_start, &session_start_str, false);
                    current_app = None;
                    session_start = None;
                    session_start_str = None;
                }
                continue;
            }

            let idle_secs = get_idle_seconds();
            let is_idle = idle_secs >= idle_threshold_secs as u64;
            let new_app = get_foreground_app();

            // Skip FocusPulse itself — never track our own process
            let new_app = new_app.filter(|a| {
                a.app_name.to_lowercase() != SELF_EXE
            });

            // If new app is ignored in DB, treat as no foreground app
            let new_app = new_app.and_then(|a| {
                let ignored = state.lock()
                    .map(|s| s.db.is_app_ignored(&a.executable_path))
                    .unwrap_or(false);
                if ignored { None } else { Some(a) }
            });

            let should_flush = match (&current_app, &new_app) {
                (Some(cur), Some(new)) => {
                    cur.executable_path != new.executable_path || is_idle
                }
                (Some(_), None) => true,
                _ => false,
            };

            if should_flush {
                flush_session(&state, &current_app, &session_start, &session_start_str, is_idle);
                current_app = None;
                session_start = None;
                session_start_str = None;
            }

            if !is_idle {
                if let Some(ref app) = new_app {
                    if current_app.is_none() {
                        session_start = Some(Instant::now());
                        session_start_str = Some(Local::now().format("%Y-%m-%dT%H:%M:%S").to_string());
                        current_app = Some(app.clone());
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
    current_app: &Option<ForegroundApp>,
    session_start: &Option<Instant>,
    session_start_str: &Option<String>,
    was_idle: bool,
) {
    if let (Some(app), Some(start), Some(start_str)) =
        (current_app, session_start, session_start_str)
    {
        let duration = start.elapsed().as_secs() as i64;
        if duration < 1 { return; }

        let end_str = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        if let Ok(s) = state.lock() {
            match s.db.upsert_app(&app.app_name, &app.executable_path) {
                Ok((app_id, is_ignored)) => {
                    // Double-check ignored flag (could have changed at runtime)
                    if !is_ignored {
                        let _ = s.db.insert_session(
                            app_id, &app.window_title,
                            start_str, &end_str,
                            duration, was_idle,
                        );
                        log::debug!("Flushed: {} ({}s idle={})", app.app_name, duration, was_idle);
                    }
                }
                Err(e) => log::error!("Failed to upsert app: {}", e),
            }
        }
    }
}
