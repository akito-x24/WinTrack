use crate::database::Database;
use anyhow::Result;

pub struct AppState {
    pub db: Database,
    pub paused: bool,
    pub current_app: Option<String>,
    pub session_start: Option<String>,
    pub is_idle: bool,
}

pub fn init_app_state() -> Result<AppState> {
    let db_path = resolve_db_path();
    log::info!("Opening database at: {}", db_path);
    let db = Database::open(&db_path)?;

    // After opening, persist the resolved path so the UI can display it
    let _ = db.update_settings(&serde_json::json!({ "database_path": db_path }));

    Ok(AppState {
        db,
        paused: false,
        current_app: None,
        session_start: None,
        is_idle: false,
    })
}

/// Resolve the DB path: prefer the value stored in settings (if the file exists),
/// otherwise fall back to the platform default.
pub fn resolve_db_path() -> String {
    // Try to read an already-configured path from settings.
    // We do a lightweight temp open just to read the setting, then reopen properly.
    let default = default_db_path();
    if let Ok(conn) = rusqlite::Connection::open(&default) {
        let _ = conn.execute_batch("
            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY DEFAULT 1,
                polling_interval_ms INTEGER NOT NULL DEFAULT 1000,
                idle_threshold_minutes INTEGER NOT NULL DEFAULT 5,
                launch_on_startup INTEGER NOT NULL DEFAULT 1,
                start_minimized INTEGER NOT NULL DEFAULT 1,
                theme TEXT NOT NULL DEFAULT 'dark',
                database_path TEXT NOT NULL DEFAULT '',
                notification_enabled INTEGER NOT NULL DEFAULT 1,
                daily_goal_minutes INTEGER NOT NULL DEFAULT 480
            );
            INSERT OR IGNORE INTO settings (id) VALUES (1);
        ");
        if let Ok(stored) = conn.query_row(
            "SELECT database_path FROM settings WHERE id = 1",
            [],
            |r| r.get::<_, String>(0),
        ) {
            if !stored.is_empty() && std::path::Path::new(&stored).parent()
                .map(|p| p.exists()).unwrap_or(false)
            {
                return stored;
            }
        }
    }
    default
}

pub fn default_db_path() -> String {
    "C:\\ProgramData\\WinTrack\\Database\\wintrack.db".to_string()
}
