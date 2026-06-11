use anyhow::{Context, Result};
use chrono::Local;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::path::Path;

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

            CREATE TABLE IF NOT EXISTS daily_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL UNIQUE,
                total_usage_seconds INTEGER NOT NULL DEFAULT 0,
                productive_seconds INTEGER NOT NULL DEFAULT 0,
                distracting_seconds INTEGER NOT NULL DEFAULT 0,
                idle_seconds INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_daily_stats_date ON daily_stats(date);

            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY DEFAULT 1,
                polling_interval_ms INTEGER NOT NULL DEFAULT 1000,
                idle_threshold_minutes INTEGER NOT NULL DEFAULT 5,
                launch_on_startup INTEGER NOT NULL DEFAULT 1,
                theme TEXT NOT NULL DEFAULT 'dark',
                database_path TEXT NOT NULL DEFAULT '',
                notification_enabled INTEGER NOT NULL DEFAULT 1,
                daily_goal_minutes INTEGER NOT NULL DEFAULT 480
            );

            INSERT OR IGNORE INTO settings (id) VALUES (1);
        ",
        )?;

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

        Ok(())
    }

    // ─── App Management ──────────────────────────────────────────────────────

    /// Insert or update app record. Returns (app_id, is_ignored).
    /// When is_ignored=true the caller must NOT save a session for this app.
    pub fn upsert_app(&self, app_name: &str, executable_path: &str) -> Result<(i64, bool)> {
        let category = auto_categorize(app_name, executable_path);

        // Insert only if not already known — never overwrite display_name or is_ignored on conflict
        self.conn.execute(
            "INSERT INTO apps (app_name, executable_path, category)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(executable_path) DO UPDATE SET
                 app_name = CASE WHEN display_name IS NULL THEN excluded.app_name ELSE app_name END",
            params![app_name, executable_path, category],
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
                    "category":                 row.get::<_, String>(4)?,
                    "is_ignored":               row.get::<_, bool>(5)?,
                    "daily_limit_minutes":      row.get::<_, Option<i64>>(6)?,
                    "reminder_interval_minutes": row.get::<_, i64>(7)?,
                    "soft_lock_enabled":        row.get::<_, bool>(8)?,
                    "total_seconds":            row.get::<_, i64>(9)?,
                    "today_seconds":            row.get::<_, i64>(10)?,
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

    pub fn get_daily_usage(&self, date: &str) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(a.display_name, a.app_name), a.executable_path, a.category,
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
                    "duration_seconds": row.get::<_, i64>(3)?,
                    "sessions":         row.get::<_, i64>(4)?,
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

        let productivity_score = if totals.0 > 0 {
            (totals.2 as f64 / totals.0 as f64 * 100.0).round() as i64
        } else {
            0
        };

        Ok(json!({
            "date": date,
            "total_active_seconds": totals.0,
            "total_idle_seconds":   totals.1,
            "productive_seconds":   totals.2,
            "productivity_score":   productivity_score,
            "apps":                 apps,
            "categories":           categories,
        }))
    }

    pub fn get_weekly_usage(&self, start_date: &str) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT date(s.start_time) as day,
                    SUM(CASE WHEN s.was_idle = 0 THEN s.duration_seconds ELSE 0 END) as active,
                    SUM(CASE WHEN s.was_idle = 1 THEN s.duration_seconds ELSE 0 END) as idle,
                    SUM(CASE WHEN s.was_idle = 0 AND a.category IN ('Productive','Development','Study') THEN s.duration_seconds ELSE 0 END) as productive
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
                    "productive_seconds": row.get::<_, i64>(3)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut app_stmt = self.conn.prepare(
            "SELECT COALESCE(a.display_name, a.app_name), a.category, SUM(s.duration_seconds) as total
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
                    "category":         row.get::<_, String>(1)?,
                    "duration_seconds": row.get::<_, i64>(2)?,
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
            "SELECT strftime('%H', s.start_time) as hour, SUM(s.duration_seconds) as total
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) = ?1 AND s.was_idle = 0 AND a.is_ignored = 0
             GROUP BY hour ORDER BY hour",
        )?;
        let hours_raw: Vec<(i32, i64)> = stmt
            .query_map(params![date], |row| {
                Ok((
                    row.get::<_, String>(0)?.parse::<i32>().unwrap_or(0),
                    row.get::<_, i64>(1)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut heatmap = vec![0i64; 24];
        for (h, secs) in hours_raw {
            if (0..24).contains(&(h as usize)) {
                heatmap[h as usize] = secs;
            }
        }
        Ok(json!({ "date": date, "hours": heatmap }))
    }

    pub fn get_timeline(&self, date: &str) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(a.display_name, a.app_name), a.category, s.window_title,
                    s.start_time, s.end_time, s.duration_seconds, s.was_idle
             FROM usage_sessions s
             JOIN apps a ON s.app_id = a.id
             WHERE date(s.start_time) = ?1 AND a.is_ignored = 0
             ORDER BY s.start_time ASC",
        )?;
        let sessions: Vec<Value> = stmt
            .query_map(params![date], |row| {
                Ok(json!({
                    "app_name":         row.get::<_, String>(0)?,
                    "category":         row.get::<_, String>(1)?,
                    "window_title":     row.get::<_, Option<String>>(2)?,
                    "start_time":       row.get::<_, String>(3)?,
                    "end_time":         row.get::<_, Option<String>>(4)?,
                    "duration_seconds": row.get::<_, i64>(5)?,
                    "was_idle":         row.get::<_, bool>(6)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(json!({ "date": date, "sessions": sessions }))
    }

    // ─── Settings ────────────────────────────────────────────────────────────

    pub fn get_settings(&self) -> Result<Value> {
        let row = self.conn.query_row(
            "SELECT polling_interval_ms, idle_threshold_minutes, launch_on_startup,
                    theme, database_path, notification_enabled, daily_goal_minutes
             FROM settings WHERE id = 1",
            [],
            |row| {
                Ok(json!({
                    "polling_interval_ms":    row.get::<_, i64>(0)?,
                    "idle_threshold_minutes": row.get::<_, i64>(1)?,
                    "launch_on_startup":      row.get::<_, bool>(2)?,
                    "theme":                  row.get::<_, String>(3)?,
                    "database_path":          row.get::<_, String>(4)?,
                    "notification_enabled":   row.get::<_, bool>(5)?,
                    "daily_goal_minutes":     row.get::<_, i64>(6)?,
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
        if let Some(v) = settings.get("theme") {
            self.conn.execute(
                "UPDATE settings SET theme = ?1 WHERE id = 1",
                params![v.as_str().unwrap_or("dark")],
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
        if let Some(v) = settings.get("database_path") {
            self.conn.execute(
                "UPDATE settings SET database_path = ?1 WHERE id = 1",
                params![v.as_str().unwrap_or("")],
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
