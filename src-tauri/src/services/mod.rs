use crate::database::Database;
use crate::monitoring::ForegroundApp;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Mirrors the monitoring loop's in-progress session so it can be flushed
/// to the database from outside the monitoring thread (e.g. when the user
/// exits from the tray, or on app shutdown). Kept in sync by the monitoring
/// loop on every tick where the foreground app changes.
#[derive(Clone)]
pub struct PendingSession {
    pub app: ForegroundApp,
    pub started_at: Instant,
    pub started_at_str: String,
}

pub struct AppState {
    pub db: Database,
    pub paused: bool,
    pub current_app: Option<String>,
    pub session_start: Option<String>,
    pub is_idle: bool,
    pub soft_lock_extensions: HashMap<(i64, String), i64>,
    pub active_soft_lock_app_ids: HashSet<i64>,
    pub pending_session: Option<PendingSession>,
}

impl AppState {
    pub fn retain_soft_lock_day(&mut self, today: &str) {
        self.soft_lock_extensions
            .retain(|(_, date), _| date == today);
    }

    pub fn soft_lock_extension_seconds(&self, app_id: i64, today: &str) -> i64 {
        self.soft_lock_extensions
            .get(&(app_id, today.to_string()))
            .copied()
            .unwrap_or(0)
    }

    pub fn grant_soft_lock_extension(&mut self, app_id: i64, today: &str, seconds: i64) {
        let extension = self
            .soft_lock_extensions
            .entry((app_id, today.to_string()))
            .or_insert(0);
        *extension += seconds;
        self.clear_soft_lock_active(app_id);
    }

    pub fn mark_soft_lock_active(&mut self, app_id: i64) -> bool {
        self.active_soft_lock_app_ids.insert(app_id)
    }

    pub fn clear_soft_lock_active(&mut self, app_id: i64) {
        self.active_soft_lock_app_ids.remove(&app_id);
    }

    pub fn has_active_soft_lock(&self, app_id: i64) -> bool {
        self.active_soft_lock_app_ids.contains(&app_id)
    }
}

pub fn init_app_state() -> Result<AppState> {
    let db_path = default_db_path();

    log::info!("Opening database at: {}", db_path);

    let db = Database::open(&db_path)?;

    Ok(AppState {
        db,
        paused: false,
        current_app: None,
        session_start: None,
        is_idle: false,
        soft_lock_extensions: HashMap::new(),
        active_soft_lock_app_ids: HashSet::new(),
        pending_session: None,
    })
}

pub fn default_db_path() -> String {
    "C:\\ProgramData\\WinTrack\\Database\\wintrack.db".to_string()
}