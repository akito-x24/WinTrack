import { invoke } from "@tauri-apps/api/core";
import { open as dialogOpen, save as dialogSave } from "@tauri-apps/plugin-dialog";
import { enable as enableAutostart, disable as disableAutostart } from "@tauri-apps/plugin-autostart";
import { sendNotification } from "@tauri-apps/plugin-notification";
import type {
  DailyStats, WeeklyStats, MonthlyStats, HourlyHeatmap,
  Timeline, Settings, CurrentSession, App,
} from "../types";

const isTauri = () => "__TAURI_INTERNALS__" in window;

async function call<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (!isTauri()) {
    throw new Error(
      "WinTrack must be run through Tauri. Use 'npm run tauri dev'."
    );
  }

  return invoke<T>(cmd, args);
}

export const api = {
  getTodayStats: () => call<DailyStats>("get_today_stats"),
  getDailyUsage: (date: string) => call<DailyStats>("get_daily_usage", { date }),
  getWeeklyUsage: (start_date: string) => call<WeeklyStats>("get_weekly_usage", { startDate: start_date }),
  getMonthlyUsage: (year: number, month: number) => call<MonthlyStats>("get_monthly_usage", { year, month }),
  getAppList: () => call<App[]>("get_app_list"),
  getHourlyHeatmap: (date: string) => call<HourlyHeatmap>("get_hourly_heatmap", { date }),
  getSettings: () => call<Settings>("get_settings"),
  getTimeline: (date: string) => call<Timeline>("get_timeline", { date }),
  getCurrentSession: () => call<CurrentSession>("get_current_session"),

  updateAppCategory: (app_id: number, category: string) => call<boolean>("update_app_category", { appId: app_id, category }),
  updateAppDisplayName: (app_id: number, display_name: string) => call<boolean>("update_app_display_name", { appId: app_id, displayName: display_name }),
  setAppIgnored: (app_id: number, ignored: boolean) => call<boolean>("set_app_ignored", { appId: app_id, ignored }),
  updateAppReminderInterval: (app_id: number, interval_minutes: number) => call<boolean>("update_app_reminder_interval", { appId: app_id, intervalMinutes: interval_minutes }),
  updateAppSoftLockEnabled: (app_id: number, enabled: boolean) => call<boolean>("set_app_soft_lock_enabled", { appId: app_id, enabled }),
  updateAppDailyLimit: (app_id: number, limit_minutes: number | null) => call<boolean>("update_app_daily_limit", { appId: app_id, limitMinutes: limit_minutes }),

  closeProcess: (process_name: string) => call<void>("close_process", { processName: process_name }),
  grantAppMoreTime: (app_id: number) => call<void>("grant_app_more_time", { appId: app_id }),
  finishSoftLockWarning: (app_id: number) => call<void>("finish_soft_lock_warning", { appId: app_id }),
  getSoftLockAppDetails: (app_id: number) => call<{ display_name: string; icon_data?: string | null }>("get_soft_lock_app_details", { appId: app_id }),

  resetTrackingData: () => call<void>("reset_tracking_data"),
  factoryReset: () => call<void>("factory_reset"),

  updateSettings: (settings: Partial<Settings>) => call<boolean>("update_settings", { settings }),
  pauseTracking: () => call<boolean>("pause_tracking"),
  resumeTracking: () => call<boolean>("resume_tracking"),
  isTrackingPaused: () => call<boolean>("is_tracking_paused"),

  exportData: (format: string, start_date: string, end_date: string, output_path: string) => call<string>("export_data", { format, startDate: start_date, endDate: end_date, outputPath: output_path }),

  async get30DayAverage(): Promise<number> { return invoke<number>("get_30_day_average"); },

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