import { invoke } from "@tauri-apps/api/core";
import {open as dialogOpen, save as dialogSave} from "@tauri-apps/plugin-dialog";
import { enable as enableAutostart, disable as disableAutostart } from "@tauri-apps/plugin-autostart";
import { sendNotification } from "@tauri-apps/plugin-notification";
import type {
  DailyStats, WeeklyStats, MonthlyStats, HourlyHeatmap,
  Timeline, Settings, CurrentSession, App,
  AppUsage, CategoryUsage
} from "../types";

const isTauri = () => "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) return getMockData(cmd, args) as T;
  return invoke<T>(cmd, args);
}

export const api = {
  getTodayStats:    () => call<DailyStats>("get_today_stats"),
  getDailyUsage:    (date: string) => call<DailyStats>("get_daily_usage", { date }),
  // Tauri v2 expects camelCase for snake_case Rust arguments
  getWeeklyUsage:   (start_date: string) => call<WeeklyStats>("get_weekly_usage", { startDate: start_date }), 
  getMonthlyUsage:  (year: number, month: number) => call<MonthlyStats>("get_monthly_usage", { year, month }),
  getAppList:       () => call<App[]>("get_app_list"),
  getHourlyHeatmap: (date: string) => call<HourlyHeatmap>("get_hourly_heatmap", { date }),
  getSettings:      () => call<Settings>("get_settings"),
  getTimeline:      (date: string) => call<Timeline>("get_timeline", { date }),
  getCurrentSession:() => call<CurrentSession>("get_current_session"),

  updateAppCategory:    (app_id: number, category: string) => call<boolean>("update_app_category", { appId: app_id, category }),
  updateAppDisplayName: (app_id: number, display_name: string) => call<boolean>("update_app_display_name", { appId: app_id, displayName: display_name }),
  setAppIgnored:        (app_id: number, ignored: boolean) => call<boolean>("set_app_ignored", { appId: app_id, ignored }),
  updateAppReminderInterval: (app_id: number, interval_minutes: number) => call<boolean>("update_app_reminder_interval", {appId: app_id, intervalMinutes: interval_minutes}),
  updateAppSoftLockEnabled: (app_id: number, enabled: boolean) => call<boolean>("set_app_soft_lock_enabled", {appId: app_id, enabled}),
  updateAppDailyLimit: (app_id: number, limit_minutes: number | null) => call<boolean>("update_app_daily_limit", {appId: app_id, limitMinutes: limit_minutes}),

  updateSettings:  (settings: Partial<Settings>) => call<boolean>("update_settings", { settings }),
  pauseTracking:   () => call<boolean>("pause_tracking"),
  resumeTracking:  () => call<boolean>("resume_tracking"),
  isTrackingPaused:() => call<boolean>("is_tracking_paused"),

  exportData:  (format: string, start_date: string, end_date: string, output_path: string) =>
    call<string>("export_data", { format, startDate: start_date, endDate: end_date, outputPath: output_path }),

  backupDatabase:    (backup_path: string) => call<boolean>("backup_database", { backupPath: backup_path }),
  moveDatabase:      (new_path: string) => call<string>("move_database", { newPath: new_path }),
  resetDatabasePath: () => call<string>("reset_database_path"),

  // ─── Autostart ────────────────────────────────────────────────────────────
  setAutostart: async (enabled: boolean): Promise<void> => {
    if (!isTauri()) return;
    try {
      if (enabled) await enableAutostart();
      else await disableAutostart();
    } catch (e) {
      console.warn("Autostart error:", e);
    }
  },

  // ─── Notifications ────────────────────────────────────────────────────────
  sendNotification: async (title: string, body: string): Promise<void> => {
    if (!isTauri()) { console.log("Notification:", title, body); return; }
    try {
      await sendNotification({ title, body });
    } catch (e) {
      console.warn("Notification error:", e);
    }
  },

  // ─── Folder picker (for DB move) ──────────────────────────────────────────
  /** Open a native folder picker and return the chosen path (or null if cancelled). */
  pickFolder: async (): Promise<string | null> => {
    if (!isTauri()) return "C:\\Users\\User\\Documents";
    try {
      const result = await dialogOpen({
        directory: true,
        multiple: false,
        title: "Select folder for database",
      });
      return typeof result === "string" ? result : null;
    } catch (e) {
      console.warn("Folder picker error:", e);
      return null;
    }
  },
  
  // ─── File save picker (for export) ────────────────────────────────────────
  pickSavePath: async (format: "csv" | "json"): Promise<string | null> => {
    if (!isTauri()) return `C:\\Users\\User\\Downloads\\wintrack-export.${format}`;
    try {
      const result = await dialogSave({
        title: "Save export file",
        defaultPath: `wintrack-export.${format}`,
        filters: format === "csv"
          ? [{ name: "CSV", extensions: ["csv"] }]
          : [{ name: "JSON", extensions: ["json"] }],
      });
      return typeof result === "string" ? result : null;
    } catch (e) {
      console.warn("Save picker error:", e);
      return null;
    }
  },
};


// ─── Mock data for browser dev ───────────────────────────────────────────────

const _initialMockApps: AppUsage[] = [
  { app_name: "VS Code",        executable_path: "C:\\VSCode\\Code.exe",    category: "Development",   duration_seconds: 7200, sessions: 3 },
  { app_name: "Google Chrome",  executable_path: "C:\\Chrome\\chrome.exe",  category: "Productive",    duration_seconds: 5400, sessions: 12 },
  { app_name: "Spotify",         executable_path: "C:\\Spotify\\spotify.exe",category: "Entertainment", duration_seconds: 3600, sessions: 2 },
  { app_name: "Discord",        executable_path: "C:\\Discord\\discord.exe",category: "Social",        duration_seconds: 1800, sessions: 5 },
  { app_name: "Steam",          executable_path: "C:\\Steam\\steam.exe",    category: "Gaming",        duration_seconds: 900,  sessions: 1 },
  { app_name: "Notion",         executable_path: "C:\\Notion\\notion.exe",  category: "Productive",    duration_seconds: 2700, sessions: 4 },
  { app_name: "Slack",          executable_path: "C:\\Slack\\slack.exe",    category: "Productive",    duration_seconds: 1200, sessions: 8 },
  { app_name: "Terminal",       executable_path: "C:\\Windows\\cmd.exe",    category: "Development",   duration_seconds: 2400, sessions: 6 },
];

let _mockAppList: (App & { total_seconds: number })[] = _initialMockApps.map((a, i) => ({
  id: i + 1,
  app_name: a.app_name,
  display_name: a.app_name,
  executable_path: a.executable_path,
  category: a.category as any,
  total_seconds: a.duration_seconds,
  is_ignored: false,
}));

function getMockData(cmd: string, args?: Record<string, unknown>): unknown {
  const today = new Date().toISOString().split("T")[0];

  const mockApps = _mockAppList.map(a => ({
    app_name: a.app_name,
    executable_path: a.executable_path,
    category: a.category as any,
    duration_seconds: a.total_seconds,
    sessions: 1,
  }));

  const totalActive = mockApps.reduce((s, a) => s + a.duration_seconds, 0);
  // const productive = mockApps
  //   .filter(a => ["Development", "Productive", "Study"].includes(a.category))
  //   .reduce((s, a) => s + a.duration_seconds, 0);

  const categories: CategoryUsage[] = [
    { category: "Development",   duration_seconds: 9600 },
    { category: "Productive",    duration_seconds: 9300 },
    { category: "Entertainment", duration_seconds: 3600 },
    { category: "Social",         duration_seconds: 1800 },
    { category: "Gaming",        duration_seconds: 900 },
  ];

  const todayStats: DailyStats = {
    date: today,
    total_active_seconds: totalActive,
    total_idle_seconds: 3600,
    apps: mockApps,
    categories,
  };

  const weekDays = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(); d.setDate(d.getDate() - (6 - i));
    const active = 14400 + Math.random() * 14400;
    return {
      date: d.toISOString().split("T")[0],
      active_seconds: Math.round(active),
      idle_seconds: Math.round(active * 0.2),
    };
  });

  switch (cmd) {
    case "get_today_stats":
    case "get_daily_usage":
      return todayStats;

    case "get_weekly_usage":
      return { start_date: args?.startDate, days: weekDays, top_apps: mockApps.slice(0, 5) };

    case "get_monthly_usage": {
      const days = Array.from({ length: 30 }, (_, i) => {
        const d = new Date(args?.year as number, (args?.month as number) - 1, i + 1);
        return { date: d.toISOString().split("T")[0], active_seconds: Math.round(10800 + Math.random() * 18000) };
      });
      return { year: args?.year, month: args?.month, days };
    }

    case "get_app_list":
      return _mockAppList.map(a => ({
        id: a.id,
        app_name: a.app_name,
        display_name: a.display_name,
         executable_path: a.executable_path,
        category: a.category as any,
        total_seconds: a.total_seconds,
        is_ignored: a.is_ignored,
      }));

    case "get_hourly_heatmap": {
      const hours = Array.from({ length: 24 }, (_, i) => {
        if (i < 7 || i > 22) return 0;
        if (i >= 9 && i <= 18) return Math.round(1800 + Math.random() * 3600);
        return Math.round(300 + Math.random() * 1200);
      });
      return { date: args?.date, hours };
    }

    case "get_timeline": {
      const sessions = _mockAppList.slice(0, 5).map((app, i) => ({
        app_name: app.app_name, category: app.category,
        window_title: `${app.app_name} - Window ${i + 1}`,
        start_time: `${today}T${String(8 + i).padStart(2, "0")}:00:00`,
        end_time:   `${today}T${String(9 + i).padStart(2, "0")}:00:00`,
        duration_seconds: 3600, was_idle: false,
      }));
      return { date: args?.date, sessions };
    }

    case "is_tracking_paused":    return false;
    case "get_current_session":   return { current_app: "VS Code", session_start: new Date().toISOString(), is_idle: false };
    case "move_database":         return args?.newPath ?? "";
    case "reset_database_path":   return "C:\\ProgramData\\WinTrack\\Database\\wintrack.db";

    case "set_app_ignored": {
      const app_id = Number(args?.appId ?? -1);
      const ignored = Boolean(args?.ignored);
      const idx = _mockAppList.findIndex(a => a.id === app_id);
      if (idx >= 0) { _mockAppList[idx].is_ignored = ignored; }
      return null;
    }
    
    case "update_app_category": {
      const app_id = Number(args?.appId ?? -1);
      const category = String(args?.category ?? "Other");
      const idx = _mockAppList.findIndex(a => a.id === app_id);
      if (idx >= 0) { _mockAppList[idx].category = category as any; }
      return null;
    }
    
    case "update_app_display_name": {
      const app_id = Number(args?.appId ?? -1);
      const display_name = String(args?.displayName ?? "");
      const idx = _mockAppList.findIndex(a => a.id === app_id);
      if (idx >= 0 && display_name) { _mockAppList[idx].display_name = display_name; }
      return null;
    }

    default: return null;
  }
}