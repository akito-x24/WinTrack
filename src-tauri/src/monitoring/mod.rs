#[cfg(target_os = "windows")]
use crate::browser_pwa::{resolve_browser_pwa, BrowserKind};
use crate::services::AppState;
use chrono::Local;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "windows")]
use windows::{
    core::{BOOL, PCWSTR, PWSTR},
    Win32::Foundation::{CloseHandle, HWND, LPARAM, PROPERTYKEY},
    Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    },
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
        EnumWindows, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible, PostMessageW, WM_CLOSE,
    },
};

#[derive(Debug, Clone)]
pub struct ForegroundApp {
    pub app_name: String,
    pub executable_path: String,
    pub window_title: String,
    pub process_name: String,
    pub monitor_bounds: Option<MonitorBounds>,
}

#[derive(Debug, Clone, Copy)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
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
        let app_user_model_id = get_window_app_user_model_id(hwnd);
        let monitor_bounds = get_window_monitor_bounds(hwnd);

        if let Some(browser) = BrowserKind::from_process_name(&exe_name) {
            if let Some(pwa) = resolve_browser_pwa(
                browser,
                &executable_path,
                app_user_model_id.as_deref(),
                &window_title,
            ) {
                return Some(ForegroundApp {
                    app_name: pwa.display_name,
                    executable_path: pwa.stable_identifier,
                    window_title,
                    process_name: exe_name,
                    monitor_bounds,
                });
            }
        }

        let (app_name, executable_path) =
            if is_uwp_host_process(&exe_name) || is_windows_apps_path(&executable_path) {
                resolve_uwp_foreground_app(app_user_model_id.as_deref(), &window_title)
                    .unwrap_or((exe_name.clone(), executable_path.clone()))
            } else {
                (exe_name.clone(), executable_path.clone())
            };

        Some(ForegroundApp {
            app_name,
            executable_path,
            window_title,
            process_name: exe_name,
            monitor_bounds,
        })
    }
}

#[cfg(target_os = "windows")]
fn get_window_monitor_bounds(hwnd: windows::Win32::Foundation::HWND) -> Option<MonitorBounds> {
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if monitor.0.is_null() {
            return None;
        }

        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            rcMonitor: Default::default(),
            rcWork: Default::default(),
            dwFlags: 0,
        };

        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return None;
        }

        Some(MonitorBounds {
            x: info.rcMonitor.left,
            y: info.rcMonitor.top,
            width: info.rcMonitor.right - info.rcMonitor.left,
            height: info.rcMonitor.bottom - info.rcMonitor.top,
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
fn is_windows_apps_path(executable_path: &str) -> bool {
    executable_path
        .to_lowercase()
        .contains(r"\program files\windowsapps\")
}

#[cfg(target_os = "windows")]
fn resolve_uwp_foreground_app(
    app_user_model_id: Option<&str>,
    window_title: &str,
) -> Option<(String, String)> {
    // UWP and packaged apps are often hosted by a generic Win32 process such as
    // ApplicationFrameHost.exe or RuntimeBroker.exe. The window itself carries
    // a package identity via Shell properties, which allows us to map the
    // foreground window back to the real app.
    let app_user_model_id = app_user_model_id?.trim();
    if !is_packaged_app_user_model_id(app_user_model_id) {
        return None;
    }

    let app_name = if !window_title.trim().is_empty() {
        window_title.to_string()
    } else {
        app_user_model_id.to_string()
    };

    Some((app_name, app_user_model_id.to_string()))
}

#[cfg(target_os = "windows")]
fn is_packaged_app_user_model_id(app_user_model_id: &str) -> bool {
    app_user_model_id.contains('!')
        && !app_user_model_id.contains('\\')
        && !app_user_model_id.contains('/')
}

#[cfg(target_os = "windows")]
fn get_window_app_user_model_id(hwnd: windows::Win32::Foundation::HWND) -> Option<String> {
    unsafe {
        let property_store: IPropertyStore = SHGetPropertyStoreForWindow(hwnd).ok()?;
        let mut app_user_model_key: PROPERTYKEY = std::mem::zeroed();

        let app_user_model_id_name: Vec<u16> = "System.AppUserModel.ID"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        PSGetPropertyKeyFromName(
            PCWSTR(app_user_model_id_name.as_ptr()),
            &mut app_user_model_key,
        )
        .ok()?;

        let propvar = property_store.GetValue(&app_user_model_key).ok()?;
        let mut buffer = [0u16; 512];
        PropVariantToString(&propvar, &mut buffer).ok()?;
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        let value = String::from_utf16_lossy(&buffer[..len]);
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    }
}

// ─── Targeted PWA window close (never taskkill /IM the whole browser) ─────
//
// PWAs are tracked as logical apps but physically run as ordinary top-level
// windows inside the shared browser process (chrome.exe / msedge.exe /
// brave.exe). Soft lock must only ever affect the window(s) belonging to the
// specific PWA whose limit was hit - never the browser process as a whole,
// which would also close every unrelated tab, window, and profile.
//
// Strategy: enumerate top-level visible windows owned by the target browser
// process, re-resolve each one's PWA identity from its own title + AppUserModelID
// (the same heuristic used for foreground tracking), and post WM_CLOSE only to
// the windows whose resolved identifier matches the target PWA. If none can be
// confidently matched, nothing is closed - callers must not fall back to
// killing the process.

#[cfg(target_os = "windows")]
struct PwaWindowSearch {
    target_identifier: String,
    browser: BrowserKind,
    matched: Vec<HWND>,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_pwa_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let search = &mut *(lparam.0 as *mut PwaWindowSearch);

    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 {
        return BOOL(1);
    }

    let exe_path = match process_executable_path(pid) {
        Some(path) => path,
        None => return BOOL(1),
    };
    let exe_name = std::path::Path::new(&exe_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if BrowserKind::from_process_name(&exe_name) != Some(search.browser) {
        return BOOL(1);
    }

    let mut title_buf = [0u16; 512];
    let title_len = GetWindowTextW(hwnd, &mut title_buf);
    let window_title = if title_len > 0 {
        String::from_utf16_lossy(&title_buf[..title_len as usize])
    } else {
        String::new()
    };

    let app_user_model_id = get_window_app_user_model_id(hwnd);

    if let Some(pwa) = resolve_browser_pwa(
        search.browser,
        &exe_path,
        app_user_model_id.as_deref(),
        &window_title,
    ) {
        if pwa.stable_identifier == search.target_identifier {
            search.matched.push(hwnd);
        }
    }

    BOOL(1)
}

#[cfg(target_os = "windows")]
fn process_executable_path(pid: u32) -> Option<String> {
    unsafe {
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

        Some(String::from_utf16_lossy(&path_buf[..path_len as usize]))
    }
}

/// Attempts to close only the window(s) belonging to a specific browser PWA.
///
/// Returns `Ok(true)` if at least one matching window was found and asked to
/// close, `Ok(false)` if no matching window could be identified (in which
/// case the caller must NOT fall back to killing the browser process), and
/// `Err` only for unexpected platform errors.
#[cfg(target_os = "windows")]
pub fn close_browser_pwa_windows(pwa_identifier: &str, process_name: &str) -> anyhow::Result<bool> {
    let browser = BrowserKind::from_process_name(process_name)
        .ok_or_else(|| anyhow::anyhow!("'{}' is not a recognized browser process", process_name))?;

    let mut search = PwaWindowSearch {
        target_identifier: pwa_identifier.to_string(),
        browser,
        matched: Vec::new(),
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_pwa_windows_proc),
            LPARAM(&mut search as *mut PwaWindowSearch as isize),
        );
    }

    if search.matched.is_empty() {
        return Ok(false);
    }

    for hwnd in search.matched {
        unsafe {
            // WM_CLOSE asks the window (and the tab/app inside it) to close
            // itself gracefully, exactly like clicking its own close button.
            // This never touches sibling windows, tabs, or browser profiles.
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, windows::Win32::Foundation::WPARAM(0), LPARAM(0));
        }
    }

    Ok(true)
}

#[cfg(not(target_os = "windows"))]
pub fn close_browser_pwa_windows(_pwa_identifier: &str, _process_name: &str) -> anyhow::Result<bool> {
    Ok(false)
}

#[cfg(not(target_os = "windows"))]
pub fn get_foreground_app() -> Option<ForegroundApp> {
    Some(ForegroundApp {
        app_name: "wintrack-dev".to_string(),
        executable_path: "/usr/bin/wintrack-dev".to_string(),
        window_title: "WinTrack Development".to_string(),
        process_name: "wintrack-dev".to_string(),
        monitor_bounds: None,
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
                    if let Ok(mut s) = state.lock() {
                        if let Ok((app_id, is_ignored)) =
                            s.db.upsert_app(&app.app_name, &app.executable_path)
                        {
                            if !is_ignored {
                                let current_session_seconds = start.elapsed().as_secs() as i64;

                                if let Ok(Some((today_usage, daily_limit))) =
                                    s.db.get_app_limit_status(app_id)
                                {
                                    let total_usage = today_usage + current_session_seconds;

                                    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                                    s.retain_soft_lock_day(&today);

                                    let limit_seconds = daily_limit * 60;
                                    let effective_limit_seconds = limit_seconds
                                        + s.soft_lock_extension_seconds(app_id, &today);
                                    if total_usage >= limit_seconds {
                                        let display_name =
                                            s.db.get_app_soft_lock_details(app_id)
                                                .map(|(name, _)| name)
                                                .unwrap_or_else(|_| app.app_name.clone());

                                        if s.db
                                            .should_send_limit_notification(app_id, &today)
                                            .unwrap_or(false)
                                        {
                                            let _ = handle
                                                .notification()
                                                .builder()
                                                .title("Daily Limit Reached")
                                                .body(format!(
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

                                                    .body(format!(
                                                        "You're still using {} after exceeding its limit.",
                                                        display_name
                                                    ))

                                                    .show();

                                            let _ = s.db.mark_reminder_sent(app_id, total_usage);

                                            println!("REAL-TIME REMINDER: {}", display_name);
                                        }

                                        if total_usage >= effective_limit_seconds
                                            && s.db.is_soft_lock_enabled(app_id).unwrap_or(false)
                                            && !s.has_active_soft_lock(app_id)
                                            && handle.get_webview_window("soft-lock").is_none()
                                            && s.mark_soft_lock_active(app_id)
                                        {
                                            println!("SOFT LOCK TRIGGERED: {}", display_name);

                                            let result = open_soft_lock_window(
                                                &handle,
                                                app_id,
                                                &display_name,
                                                &app.executable_path,
                                                &app.process_name,
                                                total_usage,
                                                limit_seconds,
                                                app.monitor_bounds,
                                            );

                                            if result.is_err() {
                                                s.clear_soft_lock_active(app_id);
                                            } else {
                                                let _ = s.db.reset_soft_lock_counter(app_id);
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

                    s.pending_session = match (&current_app, &session_start, &session_start_str) {
                        (Some(app), Some(start), Some(start_str)) if !is_idle => {
                            Some(crate::services::PendingSession {
                                app: app.clone(),
                                started_at: *start,
                                started_at_str: start_str.clone(),
                            })
                        }
                        _ => None,
                    };

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

fn open_soft_lock_window(
    handle: &AppHandle,
    app_id: i64,
    app_name: &str,
    identifier: &str,
    process_name: &str,
    current_usage_seconds: i64,
    daily_limit_seconds: i64,
    monitor_bounds: Option<MonitorBounds>,
) -> tauri::Result<()> {
    let url = format!(
        "/?softlock=1&appId={}&app={}&identifier={}&process={}&currentUsage={}&dailyLimit={}",
        app_id,
        urlencoding::encode(app_name),
        urlencoding::encode(identifier),
        urlencoding::encode(process_name),
        current_usage_seconds,
        daily_limit_seconds
    );

    let mut builder =
        WebviewWindowBuilder::new(handle, "soft-lock", tauri::WebviewUrl::App(url.into()))
            .title("WinTrack Soft Lock")
            .decorations(false)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .always_on_top(true)
            .visible_on_all_workspaces(true)
            .focused(true)
            .visible(true);

    if let Some(bounds) = monitor_bounds {
        builder = builder
            .position(bounds.x as f64, bounds.y as f64)
            .inner_size(bounds.width as f64, bounds.height as f64);
    } else {
        builder = builder.inner_size(1280.0, 720.0).center();
    }

    let window = builder.fullscreen(true).build()?;
    let _ = window.show();
    let _ = window.set_focus();

    Ok(())
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
        write_session(state, app, start_str, duration, was_idle);
    }
}

fn write_session(
    state: &Arc<Mutex<AppState>>,
    app: &ForegroundApp,
    start_str: &str,
    duration: i64,
    was_idle: bool,
) {
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

/// Flushes whatever session is currently in progress, for use by hard-exit
/// paths (tray "Exit", app shutdown) that don't have access to the
/// monitoring thread's local state. Safe to call multiple times - if there's
/// nothing pending, or it's too short to record, this is a no-op.
pub fn flush_session_for_exit(state: &Arc<Mutex<AppState>>) {
    let pending = match state.lock() {
        Ok(mut s) => s.pending_session.take(),
        Err(_) => return,
    };

    if let Some(pending) = pending {
        let duration = pending.started_at.elapsed().as_secs() as i64;
        write_session(state, &pending.app, &pending.started_at_str, duration, false);
    }
}