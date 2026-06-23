# WinTrack

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%2F11-0078D6.svg)](#getting-started)
[![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri-24C8DB.svg)](https://tauri.app/)

> Track application usage, understand your digital habits, and stay in control of your screen time - all with local-first privacy.

WinTrack is a Windows desktop application usage analytics tool built with **Tauri (Rust backend)** and **React + TypeScript frontend**, with **SQLite** for local persistence. It tracks foreground application activity - including browser PWAs and UWP/Store apps - stores everything locally, and gives you daily, weekly, monthly, timeline, and heatmap views into how you actually spend time on your PC.

If you find WinTrack useful, consider giving it a ⭐ on GitHub - it genuinely helps the project get noticed.

---

## Screenshots

> _Screenshots coming soon._

<!-- Add screenshots here once available, e.g.: -->
<!-- ![Dashboard](docs/screenshots/dashboard.png) -->
<!-- ![Weekly Analytics](docs/screenshots/weekly.png) -->
<!-- ![Timeline View](docs/screenshots/timeline.png) -->
<!-- ![Hourly Heatmap](docs/screenshots/heatmap.png) -->
<!-- ![Soft Lock Warning](docs/screenshots/softlock.png) -->

Planned screenshots:

- **Dashboard** - at-a-glance daily summary and weekly trend
- **Weekly / Monthly Analytics** - usage trends over time
- **Timeline View** - chronological session-by-session breakdown
- **Hourly Heatmap** - when during the day you're most active
- **App Breakdown** - per-app categories, limits, and reminders
- **Soft Lock Warning** - the fullscreen screen shown when a daily limit is reached

---

## Demo

> _A short demo GIF/recording will be added here._

<!-- ![Demo](docs/screenshots/demo.gif) -->

---

## Why WinTrack?

Most screen-time tools either ship your usage data to a server, lump every browser tab under one generic "Chrome" entry, or stop at showing you a chart with no way to actually act on it.

WinTrack was built to do better on all three counts:

- **Everything stays on your machine.** There's no account, no cloud sync, and no telemetry - usage data is sensitive, and the only way to guarantee it stays private is to never let it leave your device in the first place.
- **It tells apps apart properly.** A YouTube Progressive Web App and a work-related PWA both technically run inside `chrome.exe` - WinTrack resolves each one back to its real identity instead of tracking "Chrome" as one undifferentiated blob. The same applies to UWP/Microsoft Store apps hosted by generic system processes.
- **It can actually intervene, not just report.** Daily limits, recurring reminders, and a fullscreen soft-lock warning mean WinTrack can nudge you when a limit is hit - without ever resorting to anything as blunt as force-closing your entire browser.

It's a personal project, actively used and iterated on, not a polished commercial product - see [Roadmap](#roadmap) and [FAQ](#faq) for where it currently has rough edges.

---

## Features

### Usage Tracking

- Real-time foreground window detection via Win32 APIs
- Process identification, including resolving UWP/Store apps hosted by generic system processes (`ApplicationFrameHost.exe`, `RuntimeBroker.exe`) back to their real app identity
- Browser PWA detection - Chrome, Edge, and Brave PWAs (e.g. a YouTube, Spotify, Twitter or Instagram PWA) are tracked as their own distinct apps, not lumped under the browser
- Idle detection that pauses tracking after a configurable period of inactivity
- All sessions logged locally to SQLite

### Analytics

- Daily, weekly, and monthly usage views
- Timeline view - a chronological log of every tracked session
- Hourly heatmap of activity throughout the day
- Top application summaries
- 30-day rolling average

### App Controls

- Custom categories (Productive, Development, Study, Social, Entertainment, Gaming, Tools, Other)
- Custom display names per app
- Ignore toggle to exclude specific apps from tracking entirely
- Per-app daily usage limits (in 15-minute increments)
- Recurring reminder notifications once a limit is exceeded
- Soft-lock enforcement: a fullscreen warning when a daily limit is reached, with the option to take 5 more minutes or close the app. For browser PWAs, closing targets only that PWA's own window - never the entire browser, its other windows, tabs, or profiles.

### Export

- Export usage sessions to **CSV** or **JSON**
- Choose any custom date range, or use quick presets (last 7 / 30 / 90 days)

---

## Privacy

WinTrack is local-first by design - this matters more for a usage tracker than almost any other category of app, since the data involved is a direct record of your behavior.

- **All data is stored locally**, in a single SQLite database - nothing is synced anywhere.
- **No cloud sync.** There is no server component at all.
- **No telemetry.** No analytics, crash reporting, or update-check pings of any kind.
- **No account required.** There's nothing to sign up for or sign into.
- **No data leaves your device**, ever, under any feature.
- **Open source.** Every Tauri command and SQL query is visible in this repository - there's no hidden behavior to take on faith.

---

## Installation

### For most users

Download the latest installer from the [GitHub Releases](https://github.com/akito-x24/WinTrack/releases) page and run it.

- Windows installer: `.exe` (NSIS)

### For developers

See [Getting Started](#getting-started) below to build from source instead.

---

## Getting Started

### Prerequisites

- **Windows 10 or 11** - required; WinTrack depends on Win32 APIs with no cross-platform equivalent.
- **Node.js and npm** - [Node.js](https://nodejs.org/) v18+ recommended.
- **Rust and Cargo** - via [rustup](https://rustup.rs/), stable channel.
- **Git**

### Install

```bash
git clone https://github.com/akito-x24/WinTrack.git
cd WinTrack
npm install
```

### Development

- **Full desktop app (recommended):**

  ```bash
  npm run tauri dev
  ```

  Runs the actual Tauri app with the real Rust backend. Restart this command after Rust changes; frontend changes hot-reload.

- **Frontend only (browser mode):**

  ```bash
  npm run dev
  ```

  Starts the React UI alone at [http://localhost:1420](http://localhost:1420). Useful for UI-only work, but features that depend on the Tauri backend (tracking, soft lock, tray, exports) won't function in this mode.

---

## Build

```bash
npm run build        # compiles TypeScript and bundles the frontend into dist/
npm run tauri build  # packages the production Windows installer
```

`npm run tauri build` runs `npm run build` internally, so you don't need to run both separately. The installer is produced under `src-tauri/target/release/bundle/`.

### Other useful commands

| Command | Description |
|---|---|
| `cargo check` (from `src-tauri/`) | Quick Rust error check without a full build. |
| `cargo test` (from `src-tauri/`) | Runs the Rust unit test suite. |
| `cargo clippy` (from `src-tauri/`) | Rust lints. |
| `cargo fmt` (from `src-tauri/`) | Rust formatting. |

---

## Tech Stack

- **Frontend:** React, TypeScript, Vite, Tailwind CSS, Recharts, Zustand
- **Backend:** Rust, Tauri v2, Win32 APIs, SQLite (via `rusqlite`)
- **Desktop integration:** Tauri v2 plugins for autostart, notifications, shell, and dialogs

---

## Architecture

WinTrack is split into a **Rust backend** (`src-tauri/`) and a **React/TypeScript frontend** (`src/`), communicating entirely through Tauri's IPC command layer.

### Backend (Rust + Tauri)

- **`src-tauri/src/monitoring/`** - The core polling loop, run on its own thread. Uses Win32 APIs (`GetForegroundWindow`, `GetWindowTextW`, `GetLastInputInfo`, `GetWindowThreadProcessId`, and more) to detect the active window, resolve UWP host processes back to their real app via `AppUserModelID`, resolve browser PWAs via `browser_pwa`, track idle state, and flush completed sessions to the database. Also owns the soft-lock window lifecycle and the logic that safely closes a single PWA's window without touching the rest of the browser.
- **`src-tauri/src/browser_pwa.rs`** - Identifies installed Chrome/Edge/Brave PWAs and produces a stable identifier used wherever a PWA needs to be tracked, limited, or closed. Has its own unit test suite (`cargo test`).
- **`src-tauri/src/database/`** - SQLite schema, additive migrations (checked idempotently before each `ALTER TABLE`, so they're safe to run against an already-migrated database), and all query/aggregation functions.
- **`src-tauri/src/services/`** - Owns the shared, mutex-guarded `AppState`: settings cache, pause flag, soft-lock bookkeeping, and the in-progress session mirror used to flush usage data on exit so no session is lost when the app closes or the user signs out.
- **`src-tauri/src/export/`** - CSV/JSON export for a given date range.
- **`src-tauri/src/tray/`** - System tray icon, menu (Open, Pause Tracking, Resume Tracking, Exit), and their handlers.
- **`src-tauri/src/lib.rs`** - Tauri entrypoint. Registers every Tauri command, builds the tray and main window, and wires up exit handling.

### Frontend (React + TypeScript)

- Built with **Vite**, styled with **Tailwind CSS**.
- **`src/pages/`** - One component per screen: `Dashboard`, `DailyAnalytics`, `WeeklyAnalytics`, `MonthlyAnalytics`, `AppBreakdown`, `TimelineView`, `SettingsPage`, `ExportCenter`, and `SoftLockPage` (rendered in its own dedicated window, not the main app shell).
- **`src/components/charts/`** - Recharts-based and custom chart components (weekly bar chart, hourly heatmap, category pie chart).
- **`src/store/`** - A single Zustand store holding fetched stats, settings, and the current view.
- **`src/utils/api.ts`** - Thin wrapper over `@tauri-apps/api`'s `invoke`, the single bridge point between the UI and every Rust command.

### Important Files and Directories

| File / Folder | Purpose |
|---|---|
| `src-tauri/src/lib.rs` | Tauri entrypoint: window/tray setup, command registration, exit handling. |
| `src-tauri/src/monitoring/mod.rs` | Foreground/idle detection loop, UWP resolution, soft-lock window management. |
| `src-tauri/src/browser_pwa.rs` | PWA detection and identifier resolution for Chrome/Edge/Brave. |
| `src-tauri/src/database/mod.rs` | Schema, migrations, and all SQL queries. |
| `src-tauri/src/services/mod.rs` | Shared `AppState`. |
| `src-tauri/src/export/mod.rs` | CSV/JSON export. |
| `src-tauri/src/tray/mod.rs` | Tray icon, menu, and tray actions. |
| `src/pages/` | One file per screen - the main UI surfaces. |
| `src/components/` | Charts and shared UI primitives. |
| `src/utils/api.ts` | Frontend-to-backend bridge (`invoke`). |
| `src/store/index.ts` | Zustand store. |
| `src-tauri/Cargo.toml` | Rust dependencies. |
| `src-tauri/tauri.conf.json` | Window, bundle, and installer configuration. |

### Database Schema

WinTrack uses a single SQLite database at a fixed path:

```
C:\ProgramData\WinTrack\Database\wintrack.db
```

| Table | Key Columns | Notes |
|---|---|---|
| `apps` | `id`, `app_name`, `display_name`, `executable_path`, `category`, `icon_data`, `is_ignored`, `daily_limit_minutes`, `reminder_interval_minutes`, `soft_lock_enabled`, `first_seen` | One row per tracked app - a Win32 exe path, a UWP `AppUserModelID`, or a stable PWA identifier. |
| `usage_sessions` | `id`, `app_id`, `window_title`, `start_time`, `end_time`, `duration_seconds`, `was_idle` | One row per completed foreground session. |
| `settings` | `id`, `polling_interval_ms`, `idle_threshold_minutes`, `launch_on_startup`, `start_minimized`, `notification_enabled`, `daily_goal_minutes` | Singleton settings row. |
| `migrations` | `name`, `applied_at` | Tracks which schema migrations have already been applied. |

All tables are created on first run; migrations are additive only and safe to re-run.

---

## Performance Snapshot

Measured on Windows using Resource Monitor and Task Manager during normal use:

- Approximately **5 MB** private memory usage
- Up to **64 MB** working set, depending on foreground/background state
- Below **0.2% CPU** utilization during continuous background execution
- **0 B/s** network traffic during monitoring - consistent with WinTrack making no network calls at all
- Used by **25+** people during real-world testing and feedback-driven iteration

These figures come from informal profiling on the author's own setup rather than a controlled benchmark suite, and will vary by machine.

---

## Roadmap

- Broader, more robust browser/PWA detection
- Further-improved soft-lock behavior
- Additional analytics filters and summaries
- A more refined first-run onboarding and setup flow
- Automated database backups (the underlying backup logic already exists but isn't yet wired into the UI)

---

## FAQ

**Does WinTrack work offline?**
Yes, entirely. WinTrack makes no network requests of any kind - tracking, analytics, exports, and settings all run fully offline, all the time.

**Does it upload my data anywhere?**
No. There is no server, no account, and no sync feature. Every piece of data WinTrack collects stays in the local SQLite database on your machine, for as long as you keep it there.

**Can I exclude specific apps from being tracked?**
Yes - any app can be marked as ignored from the App Breakdown page, and it will be excluded from tracking and all analytics going forward.

**Will WinTrack track every app correctly?**
Most apps are detected reliably out of the box. Browser PWAs and UWP/Store apps are resolved through a set of heuristics (window titles, app identifiers, installed shortcuts) that cover the common cases well, but can occasionally misidentify an app - especially an unusual or newly installed one. If you run into an app that isn't tracked the way you'd expect, please open an issue on GitHub or reach out directly; reports like this directly shape what gets fixed next.

**Where is the database stored?**
Locally, at `C:\ProgramData\WinTrack\Database\wintrack.db`. This path is shared across all Windows user accounts on the machine.

---

## Contributing

1. Fork the repository.
2. Create a feature branch.
3. Commit your changes.
4. Open a pull request.

Before submitting a PR:

- Run `cargo fmt` and `cargo clippy` for any Rust changes.
- Run `npm run build` to confirm the frontend still type-checks and builds.
- Run `cargo test` to confirm the existing test suite still passes.

---

## License

MIT License - see [LICENSE](LICENSE) for the full text.

---

**Built with:** [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/), [React](https://reactjs.org/), and [SQLite](https://sqlite.org/) (via `rusqlite`).
