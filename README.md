# FocusPulse 🎯

> A production-grade Windows screen time tracker — fully offline, no telemetry, no cloud.

---

## Overview

FocusPulse monitors which applications you use on your Windows PC, how long you spend in each one, and presents rich analytics so you can understand and improve your digital habits.

Built with **Tauri 2 + Rust + React + TypeScript + SQLite**.

---

## Features

- ✅ Real-time foreground window detection (Win32 APIs)
- ✅ Idle detection (mouse + keyboard inactivity)
- ✅ SQLite local database — supports years of data
- ✅ System tray integration — tracks even when window is closed
- ✅ Windows startup registration
- ✅ Dashboard, Daily, Weekly, Monthly analytics
- ✅ App categorization (manual + auto-heuristics)
- ✅ Hourly heatmaps, weekly bar charts, category pie charts
- ✅ Focus streak tracking
- ✅ CSV & JSON export
- ✅ Database backup
- ✅ Fully offline — zero internet calls, zero telemetry
- ✅ Under 150MB RAM, under 2% CPU at idle

---

## Tech Stack

| Layer | Technology |
|---|---|
| UI Framework | React 18 + TypeScript |
| Desktop Shell | Tauri 2 |
| System Monitoring | Rust + Windows Win32 APIs |
| Database | SQLite (rusqlite, bundled) |
| Charts | Recharts |
| State | Zustand |
| Styling | TailwindCSS |
| Build | Vite |

---

## Folder Structure

```
focuspulse/
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── lib.rs              # App entry, Tauri commands, setup
│   │   ├── main.rs             # Binary entry point
│   │   ├── monitoring/         # Win32 foreground window + idle detection
│   │   ├── database/           # SQLite schema, queries, migrations
│   │   ├── services/           # AppState initialization
│   │   ├── tray/               # System tray event handling
│   │   ├── analytics/          # Focus streaks, derived stats
│   │   └── export/             # CSV/JSON export
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── src/                        # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── index.css
│   ├── components/
│   │   ├── charts/             # Recharts wrappers (heatmap, pie, bar)
│   │   ├── layout/             # Sidebar, Header
│   │   └── ui/                 # StatCard, AppRow, CategoryBadge, etc.
│   ├── pages/                  # Dashboard, Daily, Weekly, Monthly, etc.
│   ├── store/                  # Zustand global state
│   ├── types/                  # TypeScript interfaces
│   └── utils/                  # API bridge, helpers
├── package.json
├── vite.config.ts
├── tailwind.config.js
└── tsconfig.json
```

---

## Prerequisites

- **Windows 10/11** (monitoring uses Win32 APIs)
- **Node.js 20+**
- **Rust 1.77+** (install via https://rustup.rs)
- **Tauri CLI v2** (installed via npm)

---

## Development Setup

```bash
# 1. Clone / extract the project
cd focuspulse

# 2. Install Node dependencies
npm install

# 3. Run in development mode (hot reload)
npm run tauri dev
```

The frontend also runs standalone in a browser for UI development:
```bash
npm run dev
# Open http://localhost:1420
# Uses mock data when Tauri APIs are unavailable
```

---

## Building for Production

```bash
# Build release executable + installer
npm run tauri build
```

Output in `src-tauri/target/release/bundle/`:
- `nsis/FocusPulse_1.0.0_x64-setup.exe` — NSIS installer
- `msi/FocusPulse_1.0.0_x64_en-US.msi` — MSI installer
- `focuspulse.exe` — portable executable

---

## How It Works

### Monitoring Loop

Every 1000ms (configurable), the Rust monitoring thread:
1. Calls `GetForegroundWindow()` → gets active window handle
2. Calls `GetWindowTextW()` → gets window title
3. Calls `GetWindowThreadProcessId()` → gets process ID
4. Calls `QueryFullProcessImageNameW()` → gets full executable path
5. Calls `GetLastInputInfo()` → checks idle time
6. Compares current app with previous — if changed, **flushes the session** to SQLite
7. Sessions under 1 second are discarded

### Data Flow

```
Win32 APIs → Rust monitoring thread → AppState (Mutex)
    → SQLite (rusqlite) → Tauri IPC commands → React frontend (Zustand)
```

### Background Behavior

- Main window close → hidden to tray (not terminated)
- Tray double-click → window shown again
- Monitoring thread runs independently of UI thread
- Exit only via tray menu "Exit" or task manager

### Startup Registration

Handled by `tauri-plugin-autostart`. On Windows this writes a registry key to:
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

---

## Database Schema

```sql
apps (id, app_name, executable_path, category, first_seen)
usage_sessions (id, app_id, window_title, start_time, end_time, duration_seconds, was_idle)
daily_stats (id, date, total_usage_seconds, productive_seconds, distracting_seconds, idle_seconds)
settings (id, polling_interval_ms, idle_threshold_minutes, launch_on_startup, theme, ...)
```

Default location: `%APPDATA%\FocusPulse\focuspulse.db`

WAL journal mode enabled for concurrent read performance.

---

## Privacy

- **Zero network calls** — no DNS lookups, no HTTP requests
- **Zero telemetry** — no crash reporting, no analytics
- **All data local** — SQLite file on your machine only
- **Open architecture** — inspect every Rust command and SQL query

---

## Performance Targets

| Metric | Target |
|---|---|
| RAM usage | < 150MB |
| CPU (idle UI) | < 1% |
| CPU (monitoring) | < 0.5% |
| DB write latency | < 5ms |
| Session flush | Batched on app switch |

---

## Customization

### Polling interval
Settings → Tracking → Polling Interval (500ms – 5000ms)

### Idle threshold
Settings → Tracking → Idle Threshold (1–60 minutes)

### App categories
App Breakdown → click category badge to edit

### Adding auto-categorization rules
Edit `src-tauri/src/database/mod.rs` → `auto_categorize()` function

---

## Roadmap

- [ ] Pomodoro timer integration
- [ ] Focus mode (app blocking via Windows APIs)
- [ ] Daily usage goal notifications
- [ ] Data import from RescueTime / ActivityWatch
- [ ] Excel (.xlsx) export
- [ ] Multi-monitor window tracking
- [ ] Website tracking (browser extension companion)

---

## License

MIT
