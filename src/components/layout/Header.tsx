import { useStore } from "../../store";
import { formatDuration } from "../../utils/helpers";

const VIEW_TITLES: Record<string, string> = {
  dashboard: "Dashboard",
  daily: "Daily Analytics",
  weekly: "Weekly Analytics",
  monthly: "Monthly Analytics",
  apps: "App Breakdown",
  timeline: "Timeline",
  settings: "Settings",
  export: "Export Center",
};

export default function Header() {
  const { view, currentSession, isTrackingPaused, refreshAll } = useStore();

  return (
    <header className="h-14 flex items-center justify-between px-6 border-b border-fp-border bg-fp-surface/60 backdrop-blur-sm shrink-0">
      <h1 className="text-sm font-semibold text-fp-text">{VIEW_TITLES[view]}</h1>

      <div className="flex items-center gap-4">
        {/* Live session indicator */}
        {!isTrackingPaused && currentSession?.current_app && (
          <div className="flex items-center gap-2 bg-fp-card border border-fp-border px-3 py-1.5 rounded-full">
            <span className="w-1.5 h-1.5 rounded-full bg-fp-green animate-pulse" />
            <span className="text-xs text-fp-muted">
              {currentSession.current_app.replace(/\.exe$/i, "")}
            </span>
          </div>
        )}

        {/* Date */}
        <span className="text-xs text-fp-muted">
          {new Date().toLocaleDateString("en-US", {
            weekday: "long",
            day: "numeric",
            month: "long",
          })}
        </span>

        {/* Refresh */}
        <button
          onClick={() => refreshAll()}
          className="w-7 h-7 flex items-center justify-center rounded-lg hover:bg-fp-border text-fp-muted hover:text-fp-text transition-colors"
          title="Refresh data"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="23 4 23 10 17 10" />
            <polyline points="1 20 1 14 7 14" />
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
        </button>
      </div>
    </header>
  );
}
