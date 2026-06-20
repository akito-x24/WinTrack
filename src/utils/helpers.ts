export function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h === 0) return `${m}m`;
  if (m === 0) return `${h}h`;
  return `${h}h ${m}m`;
}

export function formatDurationVerbose(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h} hr ${m} min`;
  if (m > 0) return `${m} min ${s} sec`;
  return `${s} sec`;
} 

export function getWeekStart(date: Date = new Date()): string {
  const d = new Date(date);
  const day = d.getDay();
  const diff = d.getDate() - day + (day === 0 ? -6 : 1);
  d.setDate(diff);
  return d.toLocaleDateString("en-CA");
}

export function formatDate(dateStr: string): string {
  const d = new Date(dateStr);
  return d.toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric" });
}

export function formatTime(dateStr: string): string {
  const d = new Date(dateStr);
  return d.toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit" });
}

export function getAppIcon(appName: string): string {
  // Returns a color hex for a consistent app icon color based on name hash
  const hash = appName.split("").reduce((acc, c) => acc + c.charCodeAt(0), 0);
  const colors = [
    "#3b82f6", "#22c55e", "#f59e0b", "#8b5cf6",
    "#ef4444", "#06b6d4", "#f97316", "#ec4899",
  ];
  return colors[hash % colors.length];
}


/**
 * Returns a color from the WinTrack blue gradient based on percentage of max.
 * Matches the HourlyHeatmap color scheme.
 */
export function getHeatmapColor(secs: number, max: number): string {
  if (secs === 0) return "#1a1e28";
  const pct = secs / max;
  if (pct < 0.25) return "#284161";
  if (pct < 0.5)  return "#2550c8";
  if (pct < 0.75) return "#0749d7";
  return "#0062ff";
}

/**
 * Returns a lighter variant for "productive" overlay bars.
 */
export function getHeatmapColorLight(secs: number, max: number): string {
  if (secs === 0) return "transparent";
  const base = getHeatmapColor(secs, max);
  // Lighten the color slightly for overlay effect
  const lighten = (hex: string, amt: number) => {
    const num = parseInt(hex.replace("#", ""), 16);
    const r = Math.min(255, (num >> 16) + amt);
    const g = Math.min(255, ((num >> 8) & 0x00ff) + amt);
    const b = Math.min(255, (num & 0x0000ff) + amt);
    return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)}`;
  };
  return lighten(base, 30);
}



export function getAppInitials(appName: string): string {
  return appName
    .replace(/\.exe$/i, "")
    .split(/[.\s_-]/)
    .filter(Boolean)
    .slice(0, 2)
    .map(w => w[0]?.toUpperCase() ?? "")
    .join("");
}

export function percentOf(part: number, total: number): number {
  if (total === 0) return 0;
  return Math.round((part / total) * 100);
}

export function clamp(val: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, val));
}

export function todayString(): string {
  return new Date().toLocaleDateString("en-CA");
}

// Parses a "YYYY-MM-DD" string as local midnight (not UTC) so that
// subtracting days and re-formatting never crosses a timezone boundary.
// `new Date("YYYY-MM-DD")` parses as UTC midnight, which combined with
// local .setDate()/.toLocaleDateString() calls is the classic source of
// off-by-one-day bugs for any user not at UTC+0.
function parseLocalDateString(date: string): Date {
  const [year, month, day] = date.split("-").map(Number);
  return new Date(year, (month ?? 1) - 1, day ?? 1);
}

export function subtractDays(date: string, days: number): string {
  const d = parseLocalDateString(date);
  d.setDate(d.getDate() - days);
  return d.toLocaleDateString("en-CA");
}