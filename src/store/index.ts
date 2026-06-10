import { create } from "zustand";
import { api } from "../utils/api";
import type { DailyStats, WeeklyStats, Settings, CurrentSession, App } from "../types";
import { todayString, getWeekStart } from "../utils/helpers";

type View = "dashboard" | "daily" | "weekly" | "monthly" | "apps" | "timeline" | "settings" | "export";

interface AppStore {
  view: View;
  setView: (v: View) => void;

  todayStats: DailyStats | null;
  weeklyStats: WeeklyStats | null;
  appList: App[] | null;
  settings: Settings | null;
  currentSession: CurrentSession | null;
  isTrackingPaused: boolean;

  loading: Record<string, boolean>;
  errors: Record<string, string | null>;

  fetchTodayStats: () => Promise<void>;
  fetchWeeklyStats: () => Promise<void>;
  fetchAppList: () => Promise<void>;
  fetchSettings: () => Promise<void>;
  fetchCurrentSession: () => Promise<void>;

  updateSettings: (s: Partial<Settings>) => Promise<void>;
  updateAppCategory: (id: number, cat: string) => Promise<void>;
  updateAppDisplayName: (id: number, name: string) => Promise<void>;
  setAppIgnored: (id: number, ignored: boolean) => Promise<void>;
  updateAppDailyLimit: (id: number, limit: number | null) => Promise<void>;
  toggleTracking: () => Promise<void>;

  refreshAll: () => Promise<void>;
}

export const useStore = create<AppStore>((set, get) => ({
  view: "dashboard",
  setView: (view) => set({ view }),

  todayStats: null,
  weeklyStats: null,
  appList: null,
  settings: null,
  currentSession: null,
  isTrackingPaused: false,

  loading: {},
  errors: {},

  fetchTodayStats: async () => {
    set(s => ({ loading: { ...s.loading, today: true } }));
    try {
      set({ todayStats: await api.getTodayStats() });
    } catch (e) {
      set(s => ({ errors: { ...s.errors, today: String(e) } }));
    } finally {
      set(s => ({ loading: { ...s.loading, today: false } }));
    }
  },

  fetchWeeklyStats: async () => {
    set(s => ({ loading: { ...s.loading, weekly: true } }));
    try {
      set({ weeklyStats: await api.getWeeklyUsage(getWeekStart()) });
    } catch (e) {
      set(s => ({ errors: { ...s.errors, weekly: String(e) } }));
    } finally {
      set(s => ({ loading: { ...s.loading, weekly: false } }));
    }
  },

  fetchAppList: async () => {
    set(s => ({ loading: { ...s.loading, apps: true } }));
    try {
      const data = await api.getAppList();
      set({ appList: Array.isArray(data) ? data : [] });
    } catch (e) {
      set(s => ({ errors: { ...s.errors, apps: String(e) } }));
    } finally {
      set(s => ({ loading: { ...s.loading, apps: false } }));
    }
  },

  fetchSettings: async () => {
    try { set({ settings: await api.getSettings() }); } catch { }
  },

  fetchCurrentSession: async () => {
    try {
      const [data, paused] = await Promise.all([api.getCurrentSession(), api.isTrackingPaused()]);
      set({ currentSession: data, isTrackingPaused: paused });
    } catch { }
  },

  updateSettings: async (updates) => {
    // 1. If autostart preference is changing, call the OS API
    if (updates.launch_on_startup !== undefined) {
      try {
        await api.setAutostart(updates.launch_on_startup);
      } catch (e) {
        console.error("Failed to update autostart:", e);
      }
    }

    // 2. Save to DB
    await api.updateSettings(updates);

    // 3. Refresh state
    await get().fetchSettings();
  },


  updateAppCategory: async (id, category) => {
    await api.updateAppCategory(id, category);
    await Promise.all([get().fetchAppList(), get().fetchTodayStats()]);
  },

  updateAppDisplayName: async (id, name) => {
    await api.updateAppDisplayName(id, name);
    await Promise.all([get().fetchAppList(), get().fetchTodayStats()]);
  },

  setAppIgnored: async (id, ignored) => {
    await api.setAppIgnored(id, ignored);
    await Promise.all([get().fetchAppList(), get().fetchTodayStats()]);
  },

  updateAppDailyLimit: async (id, limit) => {
    await api.updateAppDailyLimit(id, limit);
    await Promise.all([
      get().fetchAppList(),
      get().fetchTodayStats(),
    ]);
  },

  toggleTracking: async () => {
    const paused = get().isTrackingPaused;
    if (paused) await api.resumeTracking(); else await api.pauseTracking();
    set({ isTrackingPaused: !paused });
  },

  refreshAll: async () => {
    const g = get();
    await Promise.allSettled([
      g.fetchTodayStats(),
      g.fetchWeeklyStats(),
      g.fetchCurrentSession(),
    ]);
  },
}));
