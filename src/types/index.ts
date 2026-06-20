export interface App {
  id: number;
  /** User-facing name (display_name if set, otherwise app_name) */
  display_name: string;
  /** Raw process name from OS e.g. "chrome.exe" */
  app_name: string;
  executable_path: string;
  category: AppCategory;
  is_ignored: boolean;
  
  daily_limit_minutes?: number | null;
  reminder_interval_minutes?: number;
  soft_lock_enabled?: boolean;

  total_seconds: number;
  today_seconds?: number;
  icon_data?: string | null;
}

export type AppCategory =
  | "Productive"
  | "Entertainment"
  | "Social"
  | "Gaming"
  | "Development"
  | "Study"
  | "Tools"
  | "Other";

export interface UsageSession {
  app_name: string;
  executable_path: string;
  category: AppCategory;
  icon_data?: string | null;
  window_title: string | null;
  start_time: string;
  end_time: string | null;
  duration_seconds: number;
  was_idle: boolean;
}

export interface DailyStats {
  date: string;
  total_active_seconds: number;
  total_idle_seconds: number;
  apps: AppUsage[];
  categories: CategoryUsage[];
}

export interface AppUsage {
  app_name: string;
  executable_path: string;
  category: AppCategory;
  icon_data?: string | null;
  duration_seconds: number;
  sessions: number;
}

export interface CategoryUsage {
  category: AppCategory;
  duration_seconds: number;
}

export interface WeeklyStats {
  start_date: string;
  days: DayStats[];
  top_apps: AppUsage[];
}

export interface DayStats {
  date: string;
  active_seconds: number;
  idle_seconds: number;
}

export interface MonthlyStats {
  year: number;
  month: number;
  days: { date: string; active_seconds: number }[];
}

export interface HourlyHeatmap {
  date: string;
  hours: number[];
}

export interface Timeline {
  date: string;
  sessions: UsageSession[];
}

export interface Settings {
  polling_interval_ms: number;
  idle_threshold_minutes: number;
  launch_on_startup: boolean;
  start_minimized: boolean;
  notification_enabled: boolean;
  daily_goal_minutes: number;
}

export interface CurrentSession {
  current_app: string | null;
  session_start: string | null;
  is_idle: boolean;
}

export const CATEGORY_COLORS: Record<AppCategory, string> = {
  Development: "#0066FF",   // Electric Blue
  Productive: "#FFB000",    // Neon Cyan
  Study: "#8B5CF6",         // Purple
  Social: "#FF2EA6",        // Neon Pink
  Entertainment: "#00E5FF", // Gold
  Gaming: "#FF4D00",        // Orange-Red
  Tools: "#14B8A6",         // Teal
  Other: "#94A3B8",         // Slate
};

export const CATEGORY_LABELS: AppCategory[] = [
  "Productive", "Development", "Study",
  "Entertainment", "Social", "Gaming", "Tools", "Other",
];
