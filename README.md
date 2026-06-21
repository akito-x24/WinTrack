# WinTrack 🎯

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**Executive Summary:** WinTrack is a privacy-first Windows screen time tracker built with Tauri (Rust backend + React/TypeScript frontend). It runs fully offline (no cloud or telemetry) and monitors application usage in real time. WinTrack lets you set daily usage limits, receive reminders, and view detailed analytics to understand and improve your digital habits. It is lightweight (<150 MB RAM) and designed for performance on Windows PCs.

## Key Features

- **Real-time Usage Tracking:** Monitors the foreground window using Win32 APIs and logs active sessions into SQLite. Discards very short sessions (<1 sec) or idle time.
- **Idle Detection:** Detects mouse/keyboard inactivity and pauses tracking when the user is idle.
- **Categories & Focus Streaks:** Automatically or manually categorize apps (e.g. Social, Productivity). Tracks “focus streaks” (consecutive days meeting usage goals) for motivation.
- **Analytics Dashboards:** Built-in daily, weekly, monthly views with charts (heatmaps, bar charts, pie charts) and usage breakdowns.
- **Custom Usage Goals & Reminders:** Set a daily usage goal (time limit). After reaching it, WinTrack shows configurable reminders or warnings.
- **Soft-Lock Enforcement:** (In development) After warnings, WinTrack can enforce a “soft lock” period during which limiting apps are blocked from reopening.
- **PWA / Browser App Tracking (Planned):** Future support for treating Progressive Web Apps (e.g. YouTube, ChatGPT PWAs) as separate tracked apps instead of lumping them under `chrome.exe` or `msedge.exe`.
- **System Tray & Autostart:** Runs in the system tray; can hide on close. Optionally launch on Windows startup via registry.
- **Data Export & Backup:** Export data as CSV/JSON. Automatic daily database backup to `%APPDATA%\WinTrack\` ensures persistence.
- **Fully Offline & Local:** All data is stored locally in a SQLite database. No internet calls or telemetry. You control your data.

## Architecture Overview

WinTrack is split into two main parts: a **Rust backend** (under `src-tauri/`) and a **React/TypeScript frontend** (under `src/`):

- **Backend (Rust + Tauri):** Handles Windows monitoring and data storage.  
  - `src-tauri/monitoring/`: Core loop using Win32 APIs (`GetForegroundWindow`, `GetWindowTextW`, `GetLastInputInfo`, etc.) to detect active app and idle status.  
  - `src-tauri/database/`: SQLite schema definitions, migrations, and query functions using `rusqlite`. Ensures the database is initialized on startup.  
  - `src-tauri/services/`: Manages the application state (`AppState`) including settings and focus/usage data.  
  - `src-tauri/tray/`: Manages the system tray icon and menu (show/hide window, exit).  
  - `src-tauri/analytics/`: Computes derived stats like streaks or aggregates for the frontend.  
  - `src-tauri/lib.rs`: Tauri entrypoint; it defines and registers all Tauri commands (e.g. for queries and actions from the frontend) and starts the app.  
  - The backend runs the monitoring loop on a separate thread and communicates with the frontend via Tauri IPC commands.

- **Frontend (React + TypeScript):** Provides the user interface.  
  - Uses **Vite** for fast development and build, and **Tailwind CSS** for styling.  
  - `src/pages/`: React pages (Dashboard, DailyAnalytics, WeeklyAnalytics, TimelineView, SettingsPage, etc.) displaying charts and settings.  
  - `src/components/`: Reusable UI components like charts (using **Recharts**), tables, and layout elements.  
  - `src/store/`: **Zustand** global state for UI state (e.g. current date range, filter settings).  
  - `src/utils/api.ts`: Bridges to the backend via `@tauri-apps/api`. Contains functions like `invoke("commandName")` to call Rust Tauri commands.  
  - The frontend communicates with the backend by calling Tauri commands (e.g. fetch sessions, update settings).

### Important Files and Directories

| File/Folder             | Purpose                                                                                        |
|-------------------------|------------------------------------------------------------------------------------------------|
| `src-tauri/lib.rs`      | Tauri app entry: sets up window, system tray, and registers commands (with `tauri::generate_handler!`). |
| `src-tauri/monitoring/`  | Windows monitoring code: detects active window and idle state to log usage sessions.           |
| `src-tauri/database/`   | Defines SQLite schema, runs migrations, and provides query functions for data access.          |
| `src-tauri/services/`   | Initializes and manages shared `AppState` (settings, in-memory state, etc.).                  |
| `src-tauri/tray/`       | Handles the system tray icon/menu and interactions (hide/show window, exit, etc.).            |
| `src/` (frontend)       | React frontend files (all under the `src/` directory).                                       |
| `src/pages/`            | React pages (e.g. `Dashboard.tsx`, `DailyAnalytics.tsx`, etc.) — main UI screens.             |
| `src/components/`       | Reusable UI components (charts, cards, modals, etc.).                                       |
| `src/utils/api.ts`      | Frontend API: wraps Tauri `invoke`/`call` to Rust commands (e.g. `updateAppSettings`).        |
| `src/store/`            | Zustand store definitions for managing UI state.                                            |
| `package.json` & `tsconfig.json` | Frontend config (scripts, dependencies, TypeScript).                               |
| `src-tauri/Cargo.toml`  | Rust backend config (dependencies like `rusqlite`, `windows`, Tauri).                         |
| `tauri.conf.json`       | Tauri configuration (window settings, permissions).                                         |

### Database Schema

WinTrack uses a SQLite database located by default at `%APPDATA%\WinTrack\wintrack.db`. The main tables and key columns are:

| Table             | Key Columns                                         | Description                                                 |
|-------------------|-----------------------------------------------------|-------------------------------------------------------------|
| **apps**          | `id`, `app_name`, `executable_path`, `category`, `icon_data`, `first_seen` | Registered applications. Stores display names, file paths, categories, and icon data. |
| **usage_sessions**| `id`, `app_id`, `window_title`, `start_time`, `end_time`, `duration_seconds`, `was_idle` | Logs each usage session (time spent on an app window).     |
| **settings**      | `id`, `polling_interval_ms`, `idle_threshold_minutes`, `launch_on_startup`, `start_minimized`, `notification_enabled`, `daily_goal_minutes` | Application preferences (e.g. tracking intervals, daily goal). |
| **migrations**    | `name`, `applied_at`                                | Tracks which database schema migrations have been applied.  |

All tables are created on first run. Additional indices (e.g. on timestamps) are handled in the schema to optimize queries. The app auto-applies migrations on startup if the schema is outdated.

## Getting Started (Developer Setup)

### Prerequisites

- **Windows 10/11**: Required for Win32 API monitoring.  
- **Node.js & npm**: Install [Node.js (v16+)](https://nodejs.org/). npm comes with Node.  
- **Rust & Cargo**: Install [Rust](https://rustup.rs/) (stable channel).  
- **Tauri CLI**: Install via `cargo install tauri-cli`.  
- **Git**: For cloning and version control.

### Clone and Install

```bash
# 1. Clone the repository
git clone https://github.com/akito-x24/wintrack.git
cd wintrack

# 2. Install frontend dependencies
npm install
```

### Running in Development Mode

- **Frontend only (browser mode):**  
  ```bash
  npm run dev
  ```  
  Starts the React UI at [http://localhost:1420](http://localhost:1420) (hot reload). Useful for UI work with mock data.

- **Tauri App (desktop dev):**  
  ```bash
  npm run tauri dev
  ```  
  Builds and launches the Windows desktop app. This runs both Rust and the UI. Any Rust code changes require restarting this. The window will appear; close to tray if configured.

> **Note:** The UI will display mock or empty data until the Rust backend is running. Running `npm run tauri dev` ensures full functionality.

### Build for Production

1. **Frontend build:**  
   ```bash
   npm run build
   ```
   This compiles TypeScript and bundles the frontend into `dist/`.

2. **Tauri package:**  
   ```bash
   npm run tauri build
   ```
   Generates a Windows installer (e.g. NSIS) in `src-tauri/target/release/bundle/`. For example: `WinTrack_vX.Y.Z_x64-setup.exe`.

### Common Commands

| Command                     | Description                                                        |
|-----------------------------|--------------------------------------------------------------------|
| `npm install`               | Install Node.js dependencies.                                      |
| `npm run dev`               | Run the frontend web UI (Vite dev server).                         |
| `npm run tauri dev`         | Run the full Tauri app (desktop) in dev mode.                      |
| `npm run build`             | Build the frontend for production (TypeScript compile & bundle).   |
| `npm run tauri build`       | Package the production Windows app installer (Tauri build).        |
| `cargo check`               | Check Rust code for errors without building (quick feedback).      |
| `cargo clippy --fix --allow-dirty --allow-staged` | Lint and auto-fix Rust code (requires Rust nightly or toolchain). |
| `cargo fmt -- --check`      | Check Rust code formatting (or run without `--check` to format).  |
| `npm run lint` *(if configured)* | (Optional) Lint frontend code (ESLint) if set up.               |

> **Note:** Running `npm run tauri build` automatically runs `npm run build` internally.

### Testing

- **Rust Tests:** Implement unit tests in `src-tauri/` as needed, then run `cargo test`.  
- **Frontend Tests:** Set up Jest/React Testing Library as needed (not included by default). You can add `npm test` once tests are configured.

## Usage Examples

- Set a daily goal and category in the **Settings** page.  
- View your usage in the **Dashboard** (today’s total and streak) or **Daily/Weekly** analytics pages.  
- When you exceed your goal, WinTrack will (eventually) show a warning and then start a soft-lock period (if enabled).  
- Export your data from **Settings** to CSV/JSON at any time.

## Example Code Snippets

#### Database Migration Example (Rust)

```rust
// Example: Add a new column to 'apps' if it doesn't exist
let count: i64 = self.conn.query_row(
    "SELECT COUNT(*) FROM pragma_table_info('apps') WHERE name='new_field'",
    [],
    |r| r.get(0),
)?;
if count == 0 {
    self.conn.execute_batch("ALTER TABLE apps ADD COLUMN new_field TEXT DEFAULT ''")?;
    log::info!("Migration: added apps.new_field column");
}
```

#### Tauri Command Example (Rust)

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running Tauri application");
```

#### Calling Tauri from React (TypeScript)

```tsx
import { invoke } from '@tauri-apps/api/tauri';

async function sayHello() {
  try {
    const response: string = await invoke('greet', { name: 'World' });
    console.log(response); // prints "Hello, World!"
  } catch (error) {
    console.error('Error invoking Tauri command:', error);
  }
}
```

## Contribution Guidelines

We welcome contributions! Please follow these guidelines:

- **Branching:** Use feature branches named like `feature/<feature-name>` or `bugfix/<issue>`.  
- **Commit Messages:** Use [Conventional Commits](https://www.conventionalcommits.org/) style. For example:  
  - `feat: add daily usage reminder notification`  
  - `fix: resolve unclosed delimiter error in database module`  
  - `docs: update README with new setup instructions`  
- **Pull Requests:** Open a PR with a clear title and description. Link any related issue.  
- **Code Style:**  
  - Rust: Run `cargo fmt` and `cargo clippy` before committing. Keep code idiomatic.  
  - TypeScript: Format consistently (e.g. with Prettier) and ensure code compiles (`npm run build`).  
- **Testing:** Include tests for new functionality when possible. Ensure all existing functionality still works (`cargo test`, manual checks).

## Troubleshooting

- **Rust compile error “unclosed delimiter”:** Often caused by mismatched `{}` braces. Check recent edits for missing or extra braces, especially around commented blocks. The error location (filename:line) will hint where a closing `}` is missing.
- **Stray `Ok(())` or syntax errors outside functions:** This can occur if a function or block was partially commented out. Re-examine recent changes (e.g. any `//` or removed lines) and restore or remove leftover lines.
- **Database migration errors:** If the schema gets out of sync, you may see SQL errors on startup. Try deleting or resetting the database (`wintrack.db` in `%APPDATA%\WinTrack`) to force a fresh schema creation. Ensure the `migrations` table is intact.
- **Reverting experimental changes:** A branch `backup/copilot-half-finished` contains the code state before removing the Copilot-generated feature. You can restore or compare code from that branch if needed:
  ```bash
  git checkout backup/copilot-half-finished
  ```
- **Development Build Issues:** If `npm run tauri dev` fails, try running `cargo clean` and then rebuilding. Ensure you have the required Rust toolchain (`stable`), and that `@tauri-apps/cli` is installed (`cargo install tauri-cli`).

## Changelog (Major Updates)

- **v0.1.0 (Initial Release):** Core functionality: foreground app tracking, idle detection, session logging, and basic dashboard analytics.  
- **v0.2.0:** Added app categorization, focus streak tracking, CSV/JSON export, and improvements to the UI.  
- **v1.0.0:** Stable release with performance optimizations, system tray support, autostart, and polished UX.  
- **(Experimental):** Implemented and later refactored out a “Limit Warning” feature prototype using Copilot (see backup branch).  
- **vX.Y.Z (TBD):** Future work like PWA/browser integration and full-screen enforcement to be added.

*(For detailed commit history, see the Git repository.)*

## Roadmap

- **Browser/PWA Detection:** Identify Progressive Web Apps (PWAs) and treat them as separate apps (e.g. YouTube PWA instead of `chrome.exe`). Use AppUserModelIDs, shortcuts or registry to detect installed PWAs.  
- **Fullscreen Enforcement:** Display a dedicated warning window and countdown when a usage limit is reached. Allow granting a 5-minute extension or enforcing app closure.  
- **Soft-Lock Relaunch Protection:** If a soft-lock is active, intercept app relaunch and show a lock screen or close it. (Currently, WinTrack prevents new sessions via the flag but does not intercept relaunch.)  
- **Further Features:** Possible pomodoro mode, website tracking (via browser extension), multi-monitor support, and data import from other services.

## Security & Privacy

- **Local-Only Storage:** All data is stored in a local SQLite file (`%APPDATA%\WinTrack\wintrack.db`). No data is sent over the internet.  
- **No Telemetry:** WinTrack does not include any analytics or crash-reporting services. All processing is done locally in Rust.  
- **Open Source:** You can inspect every Rust command and SQL query. No hidden behavior.  
- **Permissions:** The app only requires necessary Windows permissions (foreground window, registry access for autostart) and nothing more.

## How We Built It

WinTrack was developed using a Rust backend with the Tauri framework and a React + TypeScript frontend. We began by implementing the core monitoring loop in Rust, using the Windows API to detect active applications. We incrementally added features like the analytics dashboard, daily goals, and focus streaks. The React UI was developed using Vite and Tailwind for rapid iteration.

During development, we experimented with GitHub Copilot to draft new features. For example, a Copilot session generated a partial “limit warning” feature (including a new window and database schema changes). We carefully reviewed this output, ultimately refactoring and removing incomplete parts to keep the code clean. (The pre-cleanup code is preserved in the `backup/copilot-half-finished` branch for reference.)

**Success Criteria:** A completed WinTrack should build and run on Windows, accurately track usage, and meet performance targets (under ~150 MB RAM and minimal CPU usage). The features should function as specified: tracking & analytics working, daily limits notifying correctly, and settings/persistence reliable. Future success will include adding planned roadmap features while maintaining stability and privacy.

## License & Credits

WinTrack is open-source software. The project is currently **assumed to be MIT licensed** – please verify or replace with your chosen license. All contributions are welcome under this license. For more details, see the [LICENSE](LICENSE) file or contact the maintainers.

**Credits:** This project was built with [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/), [React](https://reactjs.org/), and [SQLite](https://sqlite.org/). Thank you to all contributors and the open-source community. 

*Please confirm any placeholders or assumptions (like the exact license or version numbers) before finalizing.*