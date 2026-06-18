use anyhow::{Context, Result};
use chrono::Local;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::path::Path;

use windows::core::{Interface, PCWSTR};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP,
};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};

use windows::Win32::UI::Shell::{
    ExtractIconExW, IShellItem, IShellItemImageFactory, SHCreateItemFromParsingName,
    SIIGBF_RESIZETOFIT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetIconInfo, PrivateExtractIconsW, HICON, ICONINFO,
};

use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;

pub struct Database {
    conn: Connection,
    pub db_path: String,
}

impl Database {
    pub fn open(db_path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open database at {}", db_path))?;
        conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=-32000;
            PRAGMA temp_store=MEMORY;
            PRAGMA mmap_size=268435456;
        ",
        )?;
        let db = Database {
            conn,
            db_path: db_path.to_string(),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        // Base schema
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS apps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_name TEXT NOT NULL,
                executable_path TEXT NOT NULL UNIQUE,
                category TEXT NOT NULL DEFAULT 'Other',
                icon_data TEXT,
                first_seen TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(executable_path)
            );

            CREATE TABLE IF NOT EXISTS usage_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_id INTEGER NOT NULL,
                window_title TEXT,
                start_time TEXT NOT NULL,
                end_time TEXT,
                duration_seconds INTEGER NOT NULL DEFAULT 0,
                was_idle INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (app_id) REFERENCES apps(id)
            ); 

            CREATE INDEX IF NOT EXISTS idx_sessions_start ON usage_sessions(start_time);
            CREATE INDEX IF NOT EXISTS idx_sessions_app_id ON usage_sessions(app_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_start_date ON usage_sessions(date(start_time));


            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY DEFAULT 1,
                polling_interval_ms INTEGER NOT NULL DEFAULT 1000,
                idle_threshold_minutes INTEGER NOT NULL DEFAULT 5,
                launch_on_startup INTEGER NOT NULL DEFAULT 1,
                start_minimized INTEGER NOT NULL DEFAULT 1,
                notification_enabled INTEGER NOT NULL DEFAULT 1,
                daily_goal_minutes INTEGER NOT NULL DEFAULT 480
            );

            INSERT OR IGNORE INTO settings (id) VALUES (1);
        ",
        )?;

        let _ = self.conn.execute(
            "ALTER TABLE settings ADD COLUMN start_minimized INTEGER NOT NULL DEFAULT 1",
            [],
        );

        // ── Migrations: add new columns if they don't exist ──────────────────
        // display_name: user-facing custom name (NULL = use app_name)
        let has_display_name: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='display_name'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if !has_display_name {
            self.conn
                .execute_batch("ALTER TABLE apps ADD COLUMN display_name TEXT;")?;
            log::info!("Migration: added apps.display_name");
        }

        let has_icon_path: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='icon_path'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if has_icon_path {
            self.conn
                .execute_batch("ALTER TABLE apps DROP COLUMN icon_path;")?;
            log::info!("Migration: removed obsolete apps.icon_path");
        }

        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS migrations (
                name TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )?;
        let refreshed_icons = self.conn.execute(
            "INSERT OR IGNORE INTO migrations (name) VALUES ('refresh_icons_v2')",
            [],
        )?;
        if refreshed_icons > 0 {
            self.conn.execute("UPDATE apps SET icon_data = NULL", [])?;
            log::info!("Migration: queued icons for high-resolution re-extraction");
        }

        self.conn.execute_batch(
            "
            UPDATE apps
                SET display_name =
                CASE
                    WHEN LOWER(app_name) LIKE '%.exe'
                    THEN SUBSTR(app_name, 1, LENGTH(app_name) - 4)
                    ELSE app_name
                END
            WHERE display_name IS NULL;
            ",
        )?;

        // is_ignored: when true, tracker skips this app entirely
        let has_ignored: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='is_ignored'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if !has_ignored {
            self.conn.execute_batch(
                "ALTER TABLE apps ADD COLUMN is_ignored INTEGER NOT NULL DEFAULT 0;",
            )?;
            log::info!("Migration: added apps.is_ignored");
        }

        // daily_limit_minutes: optional per-app limit in minutes (NULL = no limit)
        let has_daily_limit_minutes: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='daily_limit_minutes'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if !has_daily_limit_minutes {
            self.conn
                .execute_batch("ALTER TABLE apps ADD COLUMN daily_limit_minutes INTEGER;")?;
            log::info!("Migration: added apps.daily_limit_minutes");
        }

        // reminder_interval_minutes: how often to repeat reminders after limit is exceeded
        let has_reminder_interval_minutes: bool = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='reminder_interval_minutes'",
            [], |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;
        if !has_reminder_interval_minutes {
            self.conn.execute_batch(
                "ALTER TABLE apps ADD COLUMN reminder_interval_minutes INTEGER NOT NULL DEFAULT 15;"
            )?;
            log::info!("Migration: added apps.reminder_interval_minutes");
        }

        // soft_lock_enabled: whether to show the soft-lock warning window
        let has_soft_lock_enabled: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='soft_lock_enabled'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if !has_soft_lock_enabled {
            self.conn.execute_batch(
                "ALTER TABLE apps ADD COLUMN soft_lock_enabled INTEGER NOT NULL DEFAULT 0;",
            )?;
            log::info!("Migration: added apps.soft_lock_enabled");
        }

        let has_soft_lock_reminder_count: bool = self
    .conn
    .query_row(
        "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='soft_lock_reminder_count'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
    > 0;

        if !has_soft_lock_reminder_count {
            self.conn.execute_batch(
                "ALTER TABLE apps ADD COLUMN soft_lock_reminder_count INTEGER NOT NULL DEFAULT 0;",
            )?;
            log::info!("Migration: added apps.soft_lock_reminder_count");
        }

        // limit_notification_sent
        let has_limit_notification_sent: bool = self
    .conn
    .query_row(
        "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='limit_notification_sent'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
    > 0;

        if !has_limit_notification_sent {
            self.conn.execute_batch(
                "ALTER TABLE apps ADD COLUMN limit_notification_sent INTEGER NOT NULL DEFAULT 0;",
            )?;
            log::info!("Migration: added apps.limit_notification_sent");
        }

        // last_limit_notification_date
        let has_last_limit_notification_date: bool = self
    .conn
    .query_row(
        "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='last_limit_notification_date'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
    > 0;

        if !has_last_limit_notification_date {
            self.conn
                .execute_batch("ALTER TABLE apps ADD COLUMN last_limit_notification_date TEXT;")?;
            log::info!("Migration: added apps.last_limit_notification_date");
        }

        //Ignored Applications
        let _ = self.conn.execute(
            "
    UPDATE apps
    SET is_ignored = 1
    WHERE LOWER(app_name) IN (
        'explorer.exe',
        'searchhost.exe',
        'searchapp.exe',
        'textinputhost.exe',
        'widgets.exe',
        'lockapp.exe',
        'shellexperiencehost.exe',
        'runtimebroker.exe',
        'startmenuexperiencehost.exe',
        'applicationframehost.exe',
        'dwm.exe'
    )
    ",
            [],
        );

        // Newly added
        let has_last_reminder_notification_date: bool = self
    .conn
    .query_row(
        "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='last_reminder_notification_date'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
    > 0;

        if !has_last_reminder_notification_date {
            self.conn.execute_batch(
                "ALTER TABLE apps ADD COLUMN last_reminder_notification_date TEXT;",
            )?;
            log::info!("Migration: added apps.last_reminder_notification_date");
        }

        let has_last_reminder_usage_seconds: bool = self
    .conn
    .query_row(
        "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='last_reminder_usage_seconds'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
    > 0;

        if !has_last_reminder_usage_seconds {
            self.conn.execute_batch(
        "ALTER TABLE apps ADD COLUMN last_reminder_usage_seconds INTEGER NOT NULL DEFAULT 0;",
    )?;
            log::info!("Migration: added apps.last_reminder_usage_seconds");
        }

        //COMING SOON - FEATURE UNDER PROGRESS
        // Create app_locks table for tracking soft-lock lockouts
    //     let has_app_locks: bool = self
    //         .conn
    //         .query_row(
    //             "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='app_locks'",
    //             [],
    //             |r| r.get::<_, i64>(0),
    //         )
    //         .unwrap_or(0)
    //         > 0;

    //     if !has_app_locks {
    //         self.conn.execute_batch(
    //             "CREATE TABLE IF NOT EXISTS app_locks (
    //                 id INTEGER PRIMARY KEY AUTOINCREMENT,
    //                 app_id INTEGER NOT NULL,
    //                 lock_start TEXT NOT NULL DEFAULT (datetime('now')),
    //                 lock_expiration TEXT NOT NULL,
    //                 reason TEXT NOT NULL DEFAULT 'limit_reached',
    //                 FOREIGN KEY (app_id) REFERENCES apps(id)
    //             );
    //             CREATE INDEX IF NOT EXISTS idx_locks_app_id ON app_locks(app_id);
    //             CREATE INDEX IF NOT EXISTS idx_locks_expiration ON app_locks(lock_expiration);
    //             ",
    //         )?;
    //         log::info!("Migration: created app_locks table");
    //     }

    //     // Track limit reached events to prevent window spam
    //     let has_limit_reached_today: bool = self
    //         .conn
    //         .query_row(
    //             "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='limit_reached_today'",
    //             [],
    //             |r| r.get::<_, i64>(0),
    //         )
    //         .unwrap_or(0)
    //         > 0;

    //     if !has_limit_reached_today {
    //         self.conn.execute_batch(
    //             "ALTER TABLE apps ADD COLUMN limit_reached_today INTEGER NOT NULL DEFAULT 0;",
    //         )?;
    //         log::info!("Migration: added apps.limit_reached_today");
    //     }

        Ok(())
    }

    pub fn upsert_app(&self, app_name: &str, executable_path: &str) -> Result<(i64, bool)> {
        let category = auto_categorize(app_name, executable_path);
        let auto_ignored = auto_ignore_app(app_name, executable_path);

        let display_name = get_friendly_name(executable_path).unwrap_or_else(|| {
            app_name
                .strip_suffix(".exe")
                .unwrap_or(app_name)
                .to_string()
        });

        let existing_icon: Option<String> = self
            .conn
            .query_row(
                "SELECT icon_data FROM apps WHERE executable_path = ?1",
                params![executable_path],
                |row| row.get::<_, Option<String>>(0),
            )
            .unwrap_or(None);

        let icon_data = if existing_icon.is_none() {
            extract_icon_base64(executable_path).or_else(|| Some(String::new()))
        } else {
            existing_icon
        };

        // Insert only if not already known - never overwrite display_name or is_ignored on conflict
        self.conn.execute(
            "INSERT INTO apps (
                app_name, display_name, executable_path, icon_data, category, is_ignored)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(executable_path) DO UPDATE SET
                    app_name = apps.app_name,
                    icon_data = COALESCE(apps.icon_data, excluded.icon_data)",
            params![
                app_name,
                display_name,
                executable_path,
                icon_data,
                category,
                auto_ignored as i32
            ],
        )?;

        let (id, is_ignored): (i64, bool) = self.conn.query_row(
            "SELECT id, is_ignored FROM apps WHERE executable_path = ?1",
            params![executable_path],
            |row| Ok((row.get(0)?, row.get::<_, bool>(1)?)),
        )?;

        Ok((id, is_ignored))
    }

    /// Set per-app daily limit in minutes. None removes the limit.
    pub fn update_app_daily_limit(&self, app_id: i64, limit_minutes: Option<i64>) -> Result<()> {
        self.conn.execute(
            "UPDATE apps SET daily_limit_minutes = ?1 WHERE id = ?2",
            params![limit_minutes, app_id],
        )?;
        Ok(())
    }

    /// Set reminder interval in minutes for over-limit notifications.
    pub fn update_app_reminder_interval(&self, app_id: i64, interval_minutes: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE apps SET reminder_interval_minutes = ?1 WHERE id = ?2",
            params![interval_minutes, app_id],
        )?;
        Ok(())
    }

    /// Get reminder interval in minutes for an app.
    pub fn get_app_reminder_interval(&self, app_id: i64) -> Result<i64> {
        let interval = self.conn.query_row(
            "
        SELECT reminder_interval_minutes
        FROM apps
        WHERE id = ?1
        ",
            params![app_id],
            |r| r.get(0),
        )?;

        Ok(interval)
    }

    /// Enable or disable soft lock for an app.
    pub fn set_app_soft_lock_enabled(&self, app_id: i64, enabled: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE apps SET soft_lock_enabled = ?1 WHERE id = ?2",
            params![enabled as i32, app_id],
        )?;
        Ok(())
    }

    pub fn should_send_limit_notification(&self, app_id: i64, today: &str) -> Result<bool> {
        let result = self.conn.query_row(
            "SELECT
            limit_notification_sent,
            COALESCE(last_limit_notification_date, '')
         FROM apps
         WHERE id = ?1",
            params![app_id],
            |row| Ok((row.get::<_, bool>(0)?, row.get::<_, String>(1)?)),
        );

        match result {
            Ok((sent, last_date)) => Ok(!sent || last_date != today),
            Err(_) => Ok(true),
        }
    }

    pub fn mark_limit_notification_sent(&self, app_id: i64, today: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE apps
         SET limit_notification_sent = 1,
             last_limit_notification_date = ?1
         WHERE id = ?2",
            params![today, app_id],
        )?;

        Ok(())
    }

    pub fn should_send_reminder(
        &self,
        app_id: i64,
        today_usage: i64,
        reminder_interval_minutes: i64,
    ) -> Result<bool> {
        let last_usage: i64 = self.conn.query_row(
            "SELECT last_reminder_usage_seconds
         FROM apps
         WHERE id = ?1",
            params![app_id],
            |r| r.get(0),
        )?;

        Ok(today_usage - last_usage >= reminder_interval_minutes * 60)
    }

    pub fn mark_reminder_sent(&self, app_id: i64, today_usage: i64) -> Result<()> {
        self.conn.execute(
            "
        UPDATE apps
        SET last_reminder_usage_seconds = ?1
        WHERE id = ?2
        ",
            params![today_usage, app_id],
        )?;

        Ok(())
    }

    /// Check if a given executable path is ignored (fast path for monitoring loop).
    pub fn is_app_ignored(&self, executable_path: &str) -> bool {
        self.conn
            .query_row(
                "SELECT is_ignored FROM apps WHERE executable_path = ?1",
                params![executable_path],
                |r| r.get::<_, bool>(0),
            )
            .unwrap_or(false)
    }

    pub fn get_all_apps(&self) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT id,
                    COALESCE(display_name, app_name) as display_name,
                    app_name,
                    executable_path,
                    icon_data,
                    category,
                    is_ignored,
                    daily_limit_minutes,
                    reminder_interval_minutes,
                    soft_lock_enabled,
                    COALESCE(
                         (SELECT SUM(duration_seconds)
                         FROM usage_sessions
                         WHERE app_id = apps.id),
                        0
                    ) as total_seconds,

                    COALESCE(
                    (SELECT SUM(duration_seconds)
                          FROM usage_sessions
                          WHERE app_id = apps.id
                            AND DATE(start_time) = DATE('now','localtime')),
                         0
                    ) as today_seconds
             FROM apps
             ORDER BY total_seconds DESC",
        )?;
        let apps: Vec<Value> = stmt
            .query_map([], |row| {
                Ok(json!({
                    "id":                       row.get::<_, i64>(0)?,
                    "display_name":             row.get::<_, String>(1)?,
                    "app_name":                 row.get::<_, String>(2)?,
                    "executable_path":          row.get::<_, String>(3)?,
                    "icon_data":                row.get::<_, Option<String>>(4)?,
                    "category":                 row.get::<_, String>(5)?,
                    "is_ignored":               row.get::<_, bool>(6)?,
                    "daily_limit_minutes":      row.get::<_, Option<i64>>(7)?,
                    "reminder_interval_minutes": row.get::<_, i64>(8)?,
                    "soft_lock_enabled":        row.get::<_, bool>(9)?,
                    "total_seconds":            row.get::<_, i64>(10)?,
                    "today_seconds":            row.get::<_, i64>(11)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(json!(apps))
    }

    pub fn update_app_category(&self, app_id: i64, category: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE apps SET category = ?1 WHERE id = ?2",
            params![category, app_id],
        )?;
        Ok(())
    }

    /// Rename an app. Empty string is rejected.
    pub fn update_app_display_name(&self, app_id: i64, display_name: &str) -> Result<()> {
        let trimmed = display_name.trim();
        if trimmed.is_empty() {
            anyhow::bail!("Display name cannot be empty");
        }
        self.conn.execute(
            "UPDATE apps SET display_name = ?1 WHERE id = ?2",
            params![trimmed, app_id],
        )?;
        Ok(())
    }

    /// Toggle is_ignored for an app.
    pub fn set_app_ignored(&self, app_id: i64, ignored: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE apps SET is_ignored = ?1 WHERE id = ?2",
            params![ignored as i32, app_id],
        )?;
        Ok(())
    }

    // ─── Session Management ──────────────────────────────────────────────────

    pub fn insert_session(
        &self,
        app_id: i64,
        window_title: &str,
        start_time: &str,
        end_time: &str,
        duration_seconds: i64,
        was_idle: bool,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO usage_sessions (app_id, window_title, start_time, end_time, duration_seconds, was_idle)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![app_id, window_title, start_time, end_time, duration_seconds, was_idle as i32],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // ─── Stats Queries ───────────────────────────────────────────────────────
    // All queries use COALESCE(display_name, app_name) so renamed apps appear everywhere.

    pub fn get_today_stats(&self) -> Result<Value> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.get_daily_usage(&today)
    }

    pub fn get_30_day_average(&self) -> Result<i64> {
        let avg: f64 = self.conn.query_row(
            "
        SELECT COALESCE(AVG(day_total), 0)
        FROM (
            SELECT DATE(start_time) as day,
                   SUM(duration_seconds) as day_total
            FROM usage_sessions
            WHERE DATE(start_time) >= DATE('now', '-30 day')
              AND was_idle = 0
            GROUP BY DATE(start_time)
        )
        ",
            [],
            |row| row.get(0),
        )?;
        Ok(avg.round() as i64)
    }

    pub fn get_app_limit_status(&self, app_id: i64) -> Result<Option<(i64, i64)>> {
        let today_usage: i64 = self.conn.query_row(
            "
        SELECT COALESCE(SUM(duration_seconds), 0)
        FROM usage_sessions
        WHERE app_id = ?1
          AND DATE(start_time) = DATE('now','localtime')
        ",
            params![app_id],
            |r| r.get(0),
        )?;

        let daily_limit: Option<i64> = self.conn.query_row(
            "
        SELECT daily_limit_minutes
        FROM apps
        WHERE id = ?1
        ",
            params![app_id],
            |r| r.get(0),
        )?;

        match daily_limit {
            Some(limit) => Ok(Some((today_usage, limit))),
            None => Ok(None),
        }
    }

    pub fn is_soft_lock_enabled(&self, app_id: i64) -> Result<bool> {
        let enabled = self.conn.query_row(
            "
        SELECT soft_lock_enabled
        FROM apps
        WHERE id = ?1
        ",
            params![app_id],
            |r| r.get(0),
        )?;

        Ok(enabled)
    }

    pub fn get_app_display_name(&self, app_id: i64) -> Result<String> {
        let name = self.conn.query_row(
            "
        SELECT COALESCE(display_name, app_name)
        FROM apps
        WHERE id = ?1
        ",
            params![app_id],
            |r| r.get(0),
        )?;

        Ok(name)
    }

    pub fn increment_soft_lock_counter(&self, app_id: i64) -> Result<()> {
        self.conn.execute(
            "
        UPDATE apps
        SET soft_lock_reminder_count =
            soft_lock_reminder_count + 1
        WHERE id = ?1
        ",
            params![app_id],
        )?;
        Ok(())
    }

    pub fn get_soft_lock_counter(&self, app_id: i64) -> Result<i64> {
        let count = self.conn.query_row(
            "
        SELECT soft_lock_reminder_count
        FROM apps
        WHERE id = ?1
        ",
            params![app_id],
            |r| r.get(0),
        )?;
        Ok(count)
    }

    pub fn reset_soft_lock_counter(&self, app_id: i64) -> Result<()> {
        self.conn.execute(
            "
        UPDATE apps
        SET soft_lock_reminder_count = 0
        WHERE id = ?1
        ",
            params![app_id],
        )?;
        Ok(())
    }

    /// Create a lock for an app (soft-lock lockout after limit reached and app closed)
    pub fn create_app_lock(&self, app_id: i64, lock_duration_minutes: i64) -> Result<()> {
        let expiration = chrono::Local::now()
            .checked_add_signed(chrono::Duration::minutes(lock_duration_minutes))
            .ok_or_else(|| anyhow::anyhow!("Failed to calculate lock expiration"))?
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();

        self.conn.execute(
            "INSERT INTO app_locks (app_id, lock_expiration, reason) VALUES (?1, ?2, 'limit_reached')",
            params![app_id, expiration],
        )?;
        Ok(())
    }

    //COMING SOON - FEATURE UNDER PROGRESS
    /// Check if an app currently has an active lock
    pub fn get_app_lock(&self, app_id: i64) -> Result<Option<(i64, String)>> {
        let result = self.conn.query_row(
            "SELECT id, lock_expiration FROM app_locks 
             WHERE app_id = ?1 AND datetime(lock_expiration) > datetime('now')
             ORDER BY lock_expiration DESC LIMIT 1",
            params![app_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        );

        match result {
            Ok((id, expiration)) => Ok(Some((id, expiration))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Remove a specific lock
    pub fn remove_app_lock(&self, lock_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM app_locks WHERE id = ?1", params![lock_id])?;
        Ok(())
    }

    /// Clean up expired locks
    pub fn cleanup_expired_locks(&self) -> Result<()> {
        self.conn.execute(
            "DELETE FROM app_locks WHERE datetime(lock_expiration) <= datetime('now')",
            [],
        )?;
        Ok(())
    }

    /// Mark that limit was reached today for this app (prevent repeated warnings)
    pub fn mark_limit_reached_today(&self, app_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE apps SET limit_reached_today = 1 WHERE id = ?1",
            params![app_id],
        )?;
        Ok(())
    }

    /// Check if limit was already reached today for this app
    pub fn is_limit_reached_today(&self, app_id: i64) -> Result<bool> {
        let reached = self.conn.query_row(
            "SELECT limit_reached_today FROM apps WHERE id = ?1",
            params![app_id],
            |r| r.get::<_, bool>(0),
        )?;
        Ok(reached)
    }

    /// Reset daily limit reached flag (called daily)
    pub fn reset_daily_limit_flags(&self, today: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE apps SET limit_reached_today = 0 WHERE is_ignored = 0",
            [],
        )?;
        Ok(())
    }
    

    //COMING SOON - FEATURE UNDER PROGRESS
    // /// Reset limit reached flag for a specific app (after user grants 5 more minutes)
    // pub fn reset_app_limit_flag(&self, app_id: i64) -> Result<()> {
    //     self.conn.execute(
    //         "UPDATE apps SET limit_reached_today = 0 WHERE id = ?1",
    //         params![app_id],
    //     )?;
    //     Ok(())
    // }

    pub fn get_daily_usage(&self, date: &str) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(a.display_name, a.app_name), a.executable_path, a.category, a.icon_data,
                     SUM(s.duration_seconds) as total,
                     COUNT(*) as sessions
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) = ?1 AND s.was_idle = 0 AND a.is_ignored = 0
             GROUP BY a.id
             ORDER BY total DESC
             LIMIT 20",
        )?;
        let apps: Vec<Value> = stmt
            .query_map(params![date], |row| {
                Ok(json!({
                    "app_name":         row.get::<_, String>(0)?,
                    "executable_path":  row.get::<_, String>(1)?,
                    "category":         row.get::<_, String>(2)?,
                    "icon_data":        row.get::<_, Option<String>>(3)?,
                    "duration_seconds": row.get::<_, i64>(4)?,
                    "sessions":         row.get::<_, i64>(5)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let totals: (i64, i64, i64) = self.conn.query_row(
            "SELECT
                COALESCE(SUM(CASE WHEN s.was_idle = 0 THEN s.duration_seconds ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN s.was_idle = 1 THEN s.duration_seconds ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN s.was_idle = 0 AND a.category IN ('Productive','Development','Study') THEN s.duration_seconds ELSE 0 END), 0)
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) = ?1 AND a.is_ignored = 0",
            params![date],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap_or((0, 0, 0));

        let mut cat_stmt = self.conn.prepare(
            "SELECT a.category, SUM(s.duration_seconds) as total
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) = ?1 AND s.was_idle = 0 AND a.is_ignored = 0
             GROUP BY a.category
             ORDER BY total DESC",
        )?;
        let categories: Vec<Value> = cat_stmt
            .query_map(params![date], |row| {
                Ok(json!({
                    "category":         row.get::<_, String>(0)?,
                    "duration_seconds": row.get::<_, i64>(1)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(json!({
            "date": date,
            "total_active_seconds": totals.0,
            "total_idle_seconds":   totals.1,
            "apps":                 apps,
            "categories":           categories,
        }))
    }

    pub fn get_weekly_usage(&self, start_date: &str) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT date(s.start_time) as day,
                    SUM(CASE WHEN s.was_idle = 0 THEN s.duration_seconds ELSE 0 END) as active,
                    SUM(CASE WHEN s.was_idle = 1 THEN s.duration_seconds ELSE 0 END) as idle
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) >= ?1
               AND date(s.start_time) <= date(?1, '+6 days')
               AND a.is_ignored = 0
             GROUP BY day ORDER BY day",
        )?;
        let days: Vec<Value> = stmt
            .query_map(params![start_date], |row| {
                Ok(json!({
                    "date":               row.get::<_, String>(0)?,
                    "active_seconds":     row.get::<_, i64>(1)?,
                    "idle_seconds":       row.get::<_, i64>(2)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut app_stmt = self.conn.prepare(
            "SELECT COALESCE(a.display_name, a.app_name), a.executable_path, a.category, a.icon_data,
                    SUM(s.duration_seconds) as total
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) >= ?1
               AND date(s.start_time) <= date(?1, '+6 days')
               AND s.was_idle = 0 AND a.is_ignored = 0
             GROUP BY a.id ORDER BY total DESC LIMIT 10",
        )?;
        let top_apps: Vec<Value> = app_stmt
            .query_map(params![start_date], |row| {
                Ok(json!({
                    "app_name":         row.get::<_, String>(0)?,
                    "executable_path":  row.get::<_, String>(1)?,
                    "category":         row.get::<_, String>(2)?,
                    "icon_data":        row.get::<_, Option<String>>(3)?,
                    "duration_seconds": row.get::<_, i64>(4)?,
                    "sessions":         0,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(json!({ "start_date": start_date, "days": days, "top_apps": top_apps }))
    }

    pub fn get_monthly_usage(&self, year: i32, month: u32) -> Result<Value> {
        let start = format!("{:04}-{:02}-01", year, month);
        let end = format!(
            "{:04}-{:02}-{:02}",
            if month == 12 { year + 1 } else { year },
            if month == 12 { 1 } else { month + 1 },
            1
        );
        let mut stmt = self.conn.prepare(
            "SELECT date(s.start_time) as day,
                    SUM(CASE WHEN s.was_idle = 0 THEN s.duration_seconds ELSE 0 END) as active
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) >= ?1 AND date(s.start_time) < ?2 AND a.is_ignored = 0
             GROUP BY day ORDER BY day",
        )?;
        let days: Vec<Value> = stmt
            .query_map(params![start, end], |row| {
                Ok(json!({
                    "date":           row.get::<_, String>(0)?,
                    "active_seconds": row.get::<_, i64>(1)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(json!({ "year": year, "month": month, "days": days }))
    }

    pub fn get_hourly_heatmap(&self, date: &str) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT s.start_time, s.end_time
        FROM usage_sessions s
        JOIN apps a ON s.app_id = a.id
        WHERE date(s.start_time) = ?1
        AND s.was_idle = 0
        AND a.is_ignored = 0",
        )?;
        use chrono::{NaiveDateTime, Timelike};

        let rows: Vec<(String, String)> = stmt
            .query_map(params![date], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut heatmap = vec![0i64; 24];

        for (start_str, end_str) in rows {
            let start = match NaiveDateTime::parse_from_str(&start_str, "%Y-%m-%dT%H:%M:%S") {
                Ok(v) => v,
                Err(_) => continue,
            };

            let end = match NaiveDateTime::parse_from_str(&end_str, "%Y-%m-%dT%H:%M:%S") {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut current = start;

            while current < end {
                let hour = current.hour() as usize;

                let next_hour = current.date().and_hms_opt(current.hour(), 59, 59).unwrap()
                    + chrono::Duration::seconds(1);

                let segment_end = if end < next_hour { end } else { next_hour };

                let secs = (segment_end - current).num_seconds();

                heatmap[hour] += secs;

                current = segment_end;
            }
        }

        Ok(json!({ "date": date, "hours": heatmap }))
    }

    pub fn get_timeline(&self, date: &str) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(a.display_name, a.app_name), a.executable_path, a.category, a.icon_data,
                    s.window_title, s.start_time, s.end_time, s.duration_seconds, s.was_idle
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) = ?1 AND a.is_ignored = 0
             ORDER BY s.start_time ASC",
        )?;
        let sessions: Vec<Value> = stmt
            .query_map(params![date], |row| {
                Ok(json!({
                    "app_name":         row.get::<_, String>(0)?,
                    "executable_path":  row.get::<_, String>(1)?,
                    "category":         row.get::<_, String>(2)?,
                    "icon_data":        row.get::<_, Option<String>>(3)?,
                    "window_title":     row.get::<_, Option<String>>(4)?,
                    "start_time":       row.get::<_, String>(5)?,
                    "end_time":         row.get::<_, Option<String>>(6)?,
                    "duration_seconds": row.get::<_, i64>(7)?,
                    "was_idle":         row.get::<_, bool>(8)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(json!({ "date": date, "sessions": sessions }))
    }

    // ─── Settings ────────────────────────────────────────────────────────────

    pub fn get_settings(&self) -> Result<Value> {
        let row = self.conn.query_row(
            "SELECT polling_interval_ms, idle_threshold_minutes, launch_on_startup, start_minimized,
                    notification_enabled, daily_goal_minutes
             FROM settings WHERE id = 1",
            [],
            |row| {
                Ok(json!({
                    "polling_interval_ms":    row.get::<_, i64>(0)?,
                    "idle_threshold_minutes": row.get::<_, i64>(1)?,
                    "launch_on_startup":      row.get::<_, bool>(2)?,
                    "start_minimized":        row.get::<_, bool>(3)?,
                    "notification_enabled":   row.get::<_, bool>(4)?,
                    "daily_goal_minutes":     row.get::<_, i64>(5)?,
                }))
            },
        )?;
        Ok(row)
    }

    pub fn update_settings(&self, settings: &Value) -> Result<()> {
        if let Some(v) = settings.get("polling_interval_ms") {
            self.conn.execute(
                "UPDATE settings SET polling_interval_ms = ?1 WHERE id = 1",
                params![v.as_i64().unwrap_or(500)],
            )?;
        }
        if let Some(v) = settings.get("idle_threshold_minutes") {
            self.conn.execute(
                "UPDATE settings SET idle_threshold_minutes = ?1 WHERE id = 1",
                params![v.as_i64().unwrap_or(1)],
            )?;
        }
        if let Some(v) = settings.get("launch_on_startup") {
            self.conn.execute(
                "UPDATE settings SET launch_on_startup = ?1 WHERE id = 1",
                params![v.as_bool().unwrap_or(true) as i32],
            )?;
        }
        if let Some(v) = settings.get("start_minimized") {
            self.conn.execute(
                "UPDATE settings SET start_minimized = ?1 WHERE id = 1",
                params![v.as_bool().unwrap_or(false)],
            )?;
        }
        if let Some(v) = settings.get("notification_enabled") {
            self.conn.execute(
                "UPDATE settings SET notification_enabled = ?1 WHERE id = 1",
                params![v.as_bool().unwrap_or(true) as i32],
            )?;
        }
        if let Some(v) = settings.get("daily_goal_minutes") {
            self.conn.execute(
                "UPDATE settings SET daily_goal_minutes = ?1 WHERE id = 1",
                params![v.as_i64().unwrap_or(480)],
            )?;
        }
        Ok(())
    }

    pub fn get_polling_interval(&self) -> i64 {
        self.conn
            .query_row(
                "SELECT polling_interval_ms FROM settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(1000)
    }

    pub fn get_idle_threshold(&self) -> i64 {
        self.conn
            .query_row(
                "SELECT idle_threshold_minutes FROM settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(5)
    }

    // ─── Export ──────────────────────────────────────────────────────────────

    pub fn get_sessions_range(&self, start: &str, end: &str) -> Result<Vec<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(a.display_name, a.app_name), a.executable_path, a.category,
                    s.window_title, s.start_time, s.end_time, s.duration_seconds, s.was_idle
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) >= ?1 AND date(s.start_time) <= ?2
               AND a.is_ignored = 0
             ORDER BY s.start_time ASC",
        )?;
        let sessions: Vec<Value> = stmt
            .query_map(params![start, end], |row| {
                Ok(json!({
                    "app_name":         row.get::<_, String>(0)?,
                    "executable_path":  row.get::<_, String>(1)?,
                    "category":         row.get::<_, String>(2)?,
                    "window_title":     row.get::<_, Option<String>>(3)?,
                    "start_time":       row.get::<_, String>(4)?,
                    "end_time":         row.get::<_, Option<String>>(5)?,
                    "duration_seconds": row.get::<_, i64>(6)?,
                    "was_idle":         row.get::<_, bool>(7)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(sessions)
    }

    // ─── Backup / Move ────────────────────────────────────────────────────────

    pub fn backup(&self, backup_path: &str) -> Result<()> {
        if let Some(parent) = Path::new(backup_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        std::fs::copy(&self.db_path, backup_path)?;
        Ok(())
    }

    /// Copy DB file to new_path. Caller must reopen the database afterwards.
    pub fn move_to(&self, new_path: &str) -> Result<()> {
        // 🛠️ FIX: Handle case where user tries to "Move" to the exact same folder.
        if self.db_path == new_path {
            return Ok(());
        }

        if let Some(parent) = Path::new(new_path).parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create directory for {}", new_path))?;
        }
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;

        std::fs::copy(&self.db_path, new_path)
            .with_context(|| format!("Cannot copy DB to {}", new_path))?;
        Ok(())
    }

    //Reset
    pub fn reset_tracking_data(&self) -> Result<()> {
        self.conn.execute("DELETE FROM usage_sessions", [])?;
        self.conn.execute("DELETE FROM apps", [])?;
        Ok(())
    }

    pub fn factory_reset(&self) -> Result<()> {
        self.reset_tracking_data()?;

        self.conn.execute("DELETE FROM settings", [])?;

        self.conn
            .execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])?;

        Ok(())
    }
}

// ─── Auto-categorization heuristics ──────────────────────────────────────────

fn auto_categorize(app_name: &str, exe_path: &str) -> &'static str {
    let lower = app_name.to_lowercase();
    let path_lower = exe_path.to_lowercase();

    if matches_any(
        &lower,
        &[
            "code",
            "visual studio",
            "intellij",
            "pycharm",
            "rider",
            "webstorm",
            "clion",
            "goland",
            "datagrip",
            "eclipse",
            "netbeans",
            "vim",
            "neovim",
            "emacs",
            "sublime",
            "notepad++",
            "cursor",
            "windsurf",
            "zed",
        ],
    ) || matches_any(
        &path_lower,
        &[
            "\\code\\",
            "\\vscode\\",
            "jetbrains",
            "\\git\\",
            "terminal",
            "cmd.exe",
            "powershell",
        ],
    ) {
        return "Development";
    }
    if matches_any(
        &lower,
        &[
            "word",
            "excel",
            "powerpoint",
            "outlook",
            "onenote",
            "notion",
            "obsidian",
            "teams",
            "zoom",
            "meet",
            "slack",
            "trello",
            "asana",
            "jira",
            "confluence",
            "office",
            "libreoffice",
            "thunderbird",
            "calendar",
        ],
    ) {
        return "Productive";
    }
    if matches_any(
        &lower,
        &[
            "youtube",
            "netflix",
            "spotify",
            "vlc",
            "mpv",
            "plex",
            "kodi",
            "prime video",
            "disney",
            "twitch",
            "winamp",
            "foobar",
        ],
    ) {
        return "Entertainment";
    }
    if matches_any(
        &lower,
        &[
            "steam",
            "epic games",
            "battle.net",
            "origin",
            "uplay",
            "gog",
            "minecraft",
            "fortnite",
            "valorant",
            "league of legends",
            "game",
        ],
    ) || path_lower.contains("games")
        || path_lower.contains("steam")
    {
        return "Gaming";
    }
    if matches_any(
        &lower,
        &[
            "discord",
            "telegram",
            "whatsapp",
            "signal",
            "messenger",
            "twitter",
            "reddit",
            "instagram",
            "facebook",
            "linkedin",
        ],
    ) {
        return "Social";
    }
    if matches_any(
        &lower,
        &[
            "chrome", "firefox", "edge", "opera", "brave", "safari", "vivaldi",
        ],
    ) {
        return "Productive";
    }
    "Other"
}

fn matches_any(s: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| s.contains(p))
}

fn auto_ignore_app(app_name: &str, exe_path: &str) -> bool {
    let lower = app_name.to_lowercase();
    let path_lower = exe_path.to_lowercase();

    matches_any(
        &lower,
        &[
            "explorer.exe",
            "searchhost.exe",
            "searchapp.exe",
            "textinputhost.exe",
            "widgets.exe",
            "lockapp.exe",
            "shellexperiencehost.exe",
            "runtimebroker.exe",
            "startmenuexperiencehost.exe",
            "applicationframehost.exe",
            "dwm.exe",
        ],
    ) || path_lower.contains("\\windows\\systemapps\\")
}

fn get_file_description(exe_path: &str) -> Option<String> {
    let wide_path: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut handle = 0u32;

        let size = GetFileVersionInfoSizeW(PCWSTR(wide_path.as_ptr()), Some(&mut handle));

        if size == 0 {
            return None;
        }

        let mut buffer = vec![0u8; size as usize];

        if !GetFileVersionInfoW(
            PCWSTR(wide_path.as_ptr()),
            Some(0),
            size,
            buffer.as_mut_ptr() as *mut _,
        )
        .is_err()
        {
            return None;
        }

        let query: Vec<u16> = "\\StringFileInfo\\040904B0\\FileDescription"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut len = 0u32;

        if !VerQueryValueW(
            buffer.as_ptr() as *const _,
            PCWSTR(query.as_ptr()),
            &mut ptr,
            &mut len,
        )
        .as_bool()
        {
            return None;
        }

        if ptr.is_null() || len == 0 {
            return None;
        }

        let slice = std::slice::from_raw_parts(ptr as *const u16, len as usize);

        let text = String::from_utf16_lossy(slice)
            .trim_matches('\0')
            .trim()
            .to_string();

        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }
}

fn extract_icon_base64(exe_path: &str) -> Option<String> {
    if seems_uwp_package_id(exe_path) {
        extract_app_user_model_id_icon_base64(exe_path)
    } else {
        extract_executable_icon_base64(exe_path)
    }
}

fn seems_uwp_package_id(exe_path: &str) -> bool {
    exe_path.contains('!') && !exe_path.contains('\\') && !exe_path.contains('/')
}

fn extract_app_user_model_id_icon_base64(app_user_model_id: &str) -> Option<String> {
    const ICON_SIZE: i32 = 64;
    let shell_path = format!("shell:AppsFolder\\{}", app_user_model_id);
    let shell_path_w: Vec<u16> = shell_path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let shell_item: IShellItem = SHCreateItemFromParsingName(PCWSTR(shell_path_w.as_ptr()), None).ok()?;
        let image_factory: IShellItemImageFactory = shell_item.cast().ok()?;

        let bitmap = image_factory.GetImage(
            windows::Win32::Foundation::SIZE {
                cx: ICON_SIZE,
                cy: ICON_SIZE,
            },
            SIIGBF_RESIZETOFIT,
        )
        .ok()?;

        let result = encode_hbitmap_as_base64_png(bitmap);
        let _ = DeleteObject(bitmap.into());
        result
    }
}

fn extract_executable_icon_base64(exe_path: &str) -> Option<String> {
    const ICON_SIZE: i32 = 64;

    let mut icon = HICON::default();
    let wide: Vec<u16> = exe_path.encode_utf16().collect();

    if wide.len() < 260 {
        let mut path_buffer = [0u16; 260];
        path_buffer[..wide.len()].copy_from_slice(&wide);
        let mut icons = [HICON::default(); 1];
        let count = unsafe {
            PrivateExtractIconsW(
                &path_buffer,
                0,
                ICON_SIZE,
                ICON_SIZE,
                Some(&mut icons),
                None,
                0,
            )
        };
        if count > 0 && count != u32::MAX && !icons[0].is_invalid() {
            icon = icons[0];
        }
    }

    // Retain ExtractIconExW as the compatibility fallback for files that do not
    // expose a requested high-resolution icon through PrivateExtractIconsW.
    if icon.is_invalid() {
        let null_terminated: Vec<u16> = wide.iter().copied().chain(std::iter::once(0)).collect();
        let count = unsafe {
            ExtractIconExW(
                PCWSTR(null_terminated.as_ptr()),
                0,
                Some(&mut icon),
                None,
                1,
            )
        };
        if count == 0 || icon.is_invalid() {
            return None;
        }
    }

    let result = unsafe { encode_hicon_as_base64_png(icon) };
    unsafe {
        let _ = DestroyIcon(icon);
    }
    result
}

unsafe fn encode_hicon_as_base64_png(icon: HICON) -> Option<String> {
    const BYTES_PER_PIXEL: usize = 4;

    let mut icon_info = ICONINFO::default();
    GetIconInfo(icon, &mut icon_info).ok()?;

    let dc = CreateCompatibleDC(None);
    if dc.is_invalid() {
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());
        return None;
    }

    let result = (|| {
        if icon_info.hbmColor.is_invalid() {
            return None;
        }

        let mut bitmap = BITMAP::default();
        if GetObjectW(
            icon_info.hbmColor.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some((&mut bitmap as *mut BITMAP).cast()),
        ) == 0
        {
            return None;
        }

        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight;
        if width <= 0 || height <= 0 {
            return None;
        }

        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                // Negative height requests top-down rows.
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let byte_len = width as usize * height as usize * BYTES_PER_PIXEL;
        let mut bgra = vec![0u8; byte_len];
        if GetDIBits(
            dc,
            icon_info.hbmColor,
            0,
            height as u32,
            Some(bgra.as_mut_ptr().cast()),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        ) == 0
        {
            return None;
        }

        let has_alpha = bgra
            .chunks_exact(BYTES_PER_PIXEL)
            .any(|pixel| pixel[3] != 0);
        let mask = if has_alpha {
            None
        } else {
            read_icon_mask(dc, icon_info.hbmMask, width, height)
        };

        let mut rgba = Vec::with_capacity(byte_len);
        for (index, pixel) in bgra.chunks_exact(BYTES_PER_PIXEL).enumerate() {
            let alpha = if has_alpha {
                pixel[3]
            } else {
                mask.as_ref().map(|values| values[index]).unwrap_or(u8::MAX)
            };
            rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], alpha]);
        }

        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width as u32, height as u32, rgba)?;
        let mut png = Cursor::new(Vec::new());
        image.write_to(&mut png, ImageFormat::Png).ok()?;
        Some(general_purpose::STANDARD.encode(png.into_inner()))
    })();

    let _ = DeleteDC(dc);
    let _ = DeleteObject(icon_info.hbmColor.into());
    let _ = DeleteObject(icon_info.hbmMask.into());
    result
}

unsafe fn encode_hbitmap_as_base64_png(bitmap: HBITMAP) -> Option<String> {
    const BYTES_PER_PIXEL: usize = 4;

    let dc = CreateCompatibleDC(None);
    if dc.is_invalid() {
        let _ = DeleteObject(bitmap.into());
        return None;
    }

    let result = (|| {
        let mut bmp = BITMAP::default();
        if GetObjectW(
            bitmap.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some((&mut bmp as *mut BITMAP).cast()),
        ) == 0
        {
            return None;
        }

        let width = bmp.bmWidth;
        let height = bmp.bmHeight;
        if width <= 0 || height <= 0 {
            return None;
        }

        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                // Negative height requests top-down rows.
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let byte_len = width as usize * height as usize * BYTES_PER_PIXEL;
        let mut bgra = vec![0u8; byte_len];
        if GetDIBits(
            dc,
            bitmap,
            0,
            height as u32,
            Some(bgra.as_mut_ptr().cast()),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        ) == 0
        {
            return None;
        }

        let mut rgba = Vec::with_capacity(byte_len);
        for pixel in bgra.chunks_exact(BYTES_PER_PIXEL) {
            rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }

        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width as u32, height as u32, rgba)?;
        let mut png = Cursor::new(Vec::new());
        image.write_to(&mut png, ImageFormat::Png).ok()?;
        Some(general_purpose::STANDARD.encode(png.into_inner()))
    })();

    let _ = DeleteDC(dc);
    let _ = DeleteObject(bitmap.into());
    result
}

unsafe fn read_icon_mask(
    dc: windows::Win32::Graphics::Gdi::HDC,
    mask_bitmap: windows::Win32::Graphics::Gdi::HBITMAP,
    width: i32,
    height: i32,
) -> Option<Vec<u8>> {
    if mask_bitmap.is_invalid() {
        return None;
    }

    let mut mask_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut mask_pixels = vec![0u8; width as usize * height as usize * 4];
    if GetDIBits(
        dc,
        mask_bitmap,
        0,
        height as u32,
        Some(mask_pixels.as_mut_ptr().cast()),
        &mut mask_info,
        DIB_RGB_COLORS,
    ) == 0
    {
        return None;
    }

    Some(
        mask_pixels
            .chunks_exact(4)
            .map(|pixel| {
                let transparent = pixel[0] > 127 && pixel[1] > 127 && pixel[2] > 127;
                if transparent {
                    0
                } else {
                    u8::MAX
                }
            })
            .collect(),
    )
}

use std::fs;

fn get_friendly_name(executable_path: &str) -> Option<String> {
    let exe_name = Path::new(executable_path)
        .file_name()?
        .to_string_lossy()
        .to_lowercase();

    let mut folders = Vec::new();

    if let Ok(appdata) = std::env::var("APPDATA") {
        folders.push(format!(
            r"{}\Microsoft\Windows\Start Menu\Programs",
            appdata
        ));
    }

    folders.push(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs".to_string());

    for folder in folders {
        if let Some(name) = scan_start_menu_folder(&folder, &exe_name) {
            return Some(name);
        }
    }

    // None
    get_file_description(executable_path)
}

fn scan_start_menu_folder(folder: &str, exe_name: &str) -> Option<String> {
    fn recurse(dir: &Path, exe_name: &str) -> Option<String> {
        let entries = fs::read_dir(dir).ok()?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if let Some(found) = recurse(&path, exe_name) {
                    return Some(found);
                }
            }

            if path.extension().map(|e| e == "lnk").unwrap_or(false) {
                let stem = path.file_stem()?.to_string_lossy().to_string();

                let normalized = stem.replace(' ', "").to_lowercase();

                let exe_normalized = exe_name.replace(".exe", "").to_lowercase();

                if normalized.contains(&exe_normalized) || exe_normalized.contains(&normalized) {
                    return Some(stem);
                }
            }
        }

        None
    }

    recurse(Path::new(folder), exe_name)
}