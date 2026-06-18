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
    let db_path = default_db_path();

    log::info!("Opening database at: {}", db_path);

    let db = Database::open(&db_path)?;

    Ok(AppState {
        db,
        paused: false,
        current_app: None,
        session_start: None,
        is_idle: false,
    })
}

pub fn default_db_path() -> String {
    "C:\\ProgramData\\WinTrack\\Database\\wintrack.db".to_string()
}