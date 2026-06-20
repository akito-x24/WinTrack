# WinTrack

[![License: Unlicensed](https://img.shields.io/badge/License-Unresolved-lightgrey.svg)](#license--credits)

**Executive Summary:** WinTrack is a privacy-first Windows screen time tracker built with Tauri (Rust backend + React/TypeScript frontend). It runs fully offline (no cloud, no telemetry) and monitors application usage in real time, including Win32 apps, UWP/packaged apps, and browser PWAs (Chrome, Edge, Brave) as distinct tracked apps. You can set per-app daily limits with reminders and a fullscreen soft-lock enforcement screen, categorize apps, and export your data to CSV/JSON. All data lives in a single local SQLite database.

## Key Features

- **Real-time Usage Tracking:** Monitors the foreground window using Win32 APIs and logs active sessions into SQLite. Discards very short sessions (under 1 second).
- **Idle Detection:** Detects mouse/keyboard inactivity and pauses tracking when the user is idle past a configurable threshold. Idle time is not logged as a session.
- **UWP / Packaged App Tracking:** Resolves Store/UWP apps hosted by `ApplicationFrameHost.exe` or `RuntimeBroker.exe` back to their real app identity via `AppUserModelID`, instead of tracking the generic host process.
- **Browser PWA Tracking:** Detects Progressive Web Apps installed in Chrome, Edge, or Brave (e.g. a YouTube or ChatGPT PWA) and tracks each one as its own app, separate from general browser usage - including its own icon, category, and limits.
- **Categories:** Apps are auto-categorized and can be manually re-categorized (Productive, Development, Study, Social, Entertainment, Gaming, Tools, Other), with a category breakdown chart on the dashboard.
- **Per-App Daily Limits & Reminders:** Set a daily usage limit per app (in 15-minute increments) and an optional recurring reminder interval once that limit is exceeded.
- **Soft-Lock Enforcement:** When an app's daily limit is reached, WinTrack shows a fullscreen warning with the option to take 5 more minutes or close the app. For ordinary apps, "Close App" targets that process; for browser PWAs, it targets only that PWA's own window - it never closes the whole browser, all its windows, tabs, and profiles.
- **Analytics Dashboards:** Daily, weekly, and monthly views with an hourly heatmap, a weekly bar chart, and a category pie chart, all computed from local-time date boundaries so they match what you'd expect from your clock.
- **Export Center:** Export usage data as CSV or JSON for any date range, with quick presets (last 7/30/90 days) and a native save dialog.
- **System Tray & Autostart:** Runs in the system tray; closing the main window hides it rather than quitting, and tracking continues in the background. Can optionally launch on Windows startup. A manual launch always opens the main window; an autostart launch starts directly in the tray with no visible window flash.
- **Fully Offline & Local:** All data is stored locally in a single SQLite database (WAL mode). No internet calls, no telemetry, no accounts.

## Architecture Overview

WinTrack is split into a **Rust backend** (`src-tauri/`) and a **React/TypeScript frontend** (`src/`).

- **Backend (Rust + Tauri):**
  - `src-tauri/src/monitoring/`: The core polling loop. Uses Win32 APIs (`GetForegroundWindow`, `GetWindowTextW`, `GetLastInputInfo`, `GetWindowThreadProcessId`, etc.) to detect the active window, resolve UWP host processes to their real app via `AppUserModelID`, resolve browser PWAs via `browser_pwa`, track idle state, and flush completed sessions to the database. Also owns the soft-lock window lifecycle and the targeted PWA-window-close logic used by soft lock's "Close App" action.
  - `src-tauri/src/browser_pwa.rs`: Identifies installed Chrome/Edge/Brave PWAs from Start Menu shortcuts, `AppUserModelID`, and window-title heuristics, and produces a stable identifier (`wintrack-pwa://browser/profile/appid`) used everywhere a PWA needs to be tracked, limited, or safely closed.
  - `src-tauri/src/database/`: SQLite schema, additive migrations (checked idempotently via `pragma_table_info` before each `ALTER TABLE`), and all query functions, including in-memory icon extraction for Win32/UWP/PWA apps (no icon files are ever written to disk).
  - `src-tauri/src/services/`: Owns `AppState` - the shared, mutex-guarded application state (settings cache, pause flag, soft-lock bookkeeping, and the in-progress session mirror used to flush usage data on exit).
  - `src-tauri/src/export/`: CSV/JSON export for a given date range.
  - `src-tauri/src/tray/`: System tray icon, menu, and the Exit/Pause/Resume handlers.
  - `src-tauri/src/lib.rs`: Tauri entrypoint. Registers all Tauri commands, builds the tray and main window, and wires up exit/shutdown handling so the current session is flushed before the process actually terminates.
  - The monitoring loop runs on its own thread and communicates with the frontend purely through Tauri's IPC commands; there is no `analytics` module and no "focus streak" tracking - both were removed earlier in the project's history and are not part of the current feature set.

- **Frontend (React + TypeScript):**
  - Built with **Vite** and styled with **Tailwind CSS**.
  - `src/pages/`: One component per screen - `Dashboard`, `DailyAnalytics`, `WeeklyAnalytics`, `MonthlyAnalytics`, `AppBreakdown`, `TimelineView`, `SettingsPage`, `ExportCenter`, and `SoftLockPage` (rendered in its own dedicated Tauri window, not the main app shell).
  - `src/components/charts/`: Recharts/custom chart components (weekly bar chart, hourly heatmap, category pie chart).
  - `src/store/`: A single Zustand store holding fetched stats, settings, and the current view.
  - `src/utils/api.ts`: Thin wrapper over `@tauri-apps/api`'s `invoke`, plus a mock-data fallback so the UI can run in a plain browser (`npm run dev`) without the Rust backend.
  - `SettingsPage` and `ExportCenter` are loaded via `React.lazy()` since they aren't needed on first paint.

### Important Files and Directories

| File/Folder | Purpose |
|---|---|
| `src-tauri/src/lib.rs` | Tauri entrypoint: window/tray setup, command registration, exit handling. |
| `src-tauri/src/monitoring/mod.rs` | Foreground/idle detection loop, UWP resolution, soft-lock window + targeted PWA close. |
| `src-tauri/src/browser_pwa.rs` | PWA detection and stable identifier resolution for Chrome/Edge/Brave. |
| `src-tauri/src/database/mod.rs` | Schema, migrations, all SQL queries, icon extraction. |
| `src-tauri/src/services/mod.rs` | Shared `AppState`, including the pending-session mirror used to flush on exit. |
| `src-tauri/src/export/mod.rs` | CSV/JSON export. |
| `src-tauri/src/tray/mod.rs` | Tray icon, menu, Exit/Pause/Resume. |
| `src/pages/` | One file per screen - the main UI surfaces. |
| `src/pages/SoftLockPage.tsx` | The fullscreen daily-limit warning window. |
| `src/components/` | Charts and shared UI primitives. |
| `src/utils/api.ts` | Frontend-to-backend bridge (`invoke`) plus dev-mode mock data. |
| `src/utils/helpers.ts` | Date/duration formatting helpers - all date math is done in local time to match the backend. |
| `src/store/index.ts` | Zustand store. |
| `src-tauri/Cargo.toml` | Rust dependencies. |
| `src-tauri/tauri.conf.json` | Window, bundle, and installer configuration. |

### Database Schema

WinTrack uses a single SQLite database (WAL mode) at a fixed, non-user-selectable path:

```
C:\ProgramData\WinTrack\Database\wintrack.db
```

This path is shared by all Windows user accounts on the machine - see [Known Limitations](#known-limitations) below.

| Table | Key Columns | Notes |
|---|---|---|
| `apps` | `id`, `app_name`, `executable_path` (unique key), `display_name`, `category`, `icon_data`, `is_ignored`, `daily_limit_minutes`, `reminder_interval_minutes`, `soft_lock_enabled`, `soft_lock_reminder_count`, `limit_notification_sent`, `last_limit_notification_date`, `last_reminder_notification_date`, `last_reminder_usage_seconds`, `first_seen` | One row per tracked app (Win32 exe path, UWP `AppUserModelID`, or `wintrack-pwa://...` identifier for a PWA). Per-app settings live here, which is why "Reset Data" clears them too (see [Known Limitations](#known-limitations)). |
| `usage_sessions` | `id`, `app_id`, `window_title`, `start_time`, `end_time`, `duration_seconds`, `was_idle` | One row per completed foreground session. `was_idle` is always `0` in practice today - idle time is paused, not logged as its own session. |
| `settings` | `id`, `polling_interval_ms`, `idle_threshold_minutes`, `launch_on_startup`, `start_minimized`, `notification_enabled`, `daily_goal_minutes` | Singleton row (`id = 1`). `daily_goal_minutes` has no corresponding UI control today - see [Known Limitations](#known-limitations). |
| `migrations` | `name`, `applied_at` | Tracks which additive schema migrations have already run. |

All tables are created on first run; migrations are additive only (no down-migrations) and check `pragma_table_info` before altering, so re-running them on an already-migrated database is a no-op.

## Getting Started (Developer Setup)

### Prerequisites

- **Windows 10/11** - required for the Win32 foreground/idle/icon APIs this app depends on.
- **Node.js & npm** - [Node.js](https://nodejs.org/) v18+ recommended.
- **Rust & Cargo** - [rustup](https://rustup.rs/), stable channel.
- **Tauri CLI** - `cargo install tauri-cli`, or use the `npm run tauri` scripts below (they invoke `@tauri-apps/cli` from `devDependencies`).
- **Git**.

### Clone and Install

```bash
git clone https://github.com/yourusername/wintrack.git
cd wintrack
npm install
```

### Running in Development Mode

- **Frontend only (browser mode):**
  ```bash
  npm run dev
  ```
  Starts the React UI at [http://localhost:1420](http://localhost:1420) with mock data (see `src/utils/api.ts`). Useful for UI-only work, but soft lock, real tracking, and tray behavior aren't exercised this way.

- **Full Tauri app (desktop dev):**
  ```bash
  npm run tauri dev
  ```
  Builds and runs the actual Windows desktop app, Rust backend included. Rust changes require restarting this command; frontend changes hot-reload.

### Build for Production

```bash
npm run build        # compiles TypeScript and bundles the frontend into dist/
npm run tauri build  # packages the production Windows installer
```

`npm run tauri build` runs `npm run build` internally, so you don't need to run them separately. The installer (NSIS) is produced under `src-tauri/target/release/bundle/`.

### Common Commands

| Command | Description |
|---|---|
| `npm install` | Install Node.js dependencies. |
| `npm run dev` | Frontend-only dev server with mock data. |
| `npm run tauri dev` | Full desktop app in dev mode. |
| `npm run build` | Production frontend build. |
| `npm run tauri build` | Production Windows installer. |
| `cargo check` (from `src-tauri/`) | Quick Rust error check without a full build. |
| `cargo clippy` (from `src-tauri/`) | Rust lints. |
| `cargo fmt` (from `src-tauri/`) | Rust formatting. |

### Testing

- **Rust:** `browser_pwa.rs` ships its own unit tests; run them with `cargo test` from `src-tauri/`.
- **Frontend:** no test runner is configured by default.

## Usage Notes

- Set a per-app daily limit and (optionally) a reminder interval from the **App Breakdown** page.
- View usage on the **Dashboard**, or drill into **Daily / Weekly / Monthly** analytics.
- When an app exceeds its daily limit, WinTrack shows the soft-lock fullscreen warning if soft lock is enabled for that app, with a 30-second countdown, a "Close App" action, and a "+5 minutes" extension.
- Export data from **Export Center** (its own page, not Settings) to CSV or JSON for any date range.

## Known Limitations

These are accepted, current behaviors of the app - not regressions, and not on the near-term roadmap:

- **Shared database, no per-user separation.** The database path is fixed and the installer mode is per-machine, so on a shared Windows PC, all accounts' usage merges into one database.
- **"Reset Data" also clears per-app settings.** Because categories, limits, reminders, and ignore flags live on the `apps` table, deleting tracking history currently deletes that configuration too, even though it's framed as "delete history while keeping settings."
- **No automatic backup.** A `Database::backup()`/`move_to()` implementation exists but isn't wired into the UI; if `wintrack.db` is lost or corrupted, recovery depends on your own backups.
- **PWA identification is heuristic.** PWAs are matched by Start Menu shortcuts, `AppUserModelID`, and window-title matching, in that priority order. This is robust for the common case but can occasionally misattribute a PWA if shortcuts are unusual, or re-identify it as a new app if it's reinstalled under a different browser profile.
- **The "Daily Goal" setting has no UI.** `settings.daily_goal_minutes` exists in the schema and is read/written by the settings commands, but no page exposes a control for it, and the monitoring loop doesn't use it for notifications (limits are per-app only).
- **Windows-only.** This is by design - the Win32 foreground/idle/icon APIs and registry-based autostart this app relies on don't have cross-platform equivalents.

## Security & Privacy

- **Local-only storage.** Everything lives in one local SQLite file. WinTrack makes no network requests.
- **No telemetry.** No analytics, crash reporting, or update-check pings.
- **Open source.** Every Tauri command and SQL query is visible in this repository.

## License & Credits

WinTrack does not currently have a published license file, and its licensing status is unresolved. Treat the source as "all rights reserved" until a `LICENSE` file is added to this repository; do not assume MIT or any other license is in effect.

**Built with:** [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/), [React](https://reactjs.org/), [SQLite](https://sqlite.org/) (via `rusqlite`), and [Recharts](https://recharts.org/).
