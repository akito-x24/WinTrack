import clsx from "clsx";
import { useStore } from "../../store";
import logo from "../../assets/logo.png";

type NavItem = {
  id: string;
  label: string;
  icon: string;
};

const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "🏠︎" },
  { id: "daily", label: "Daily", icon: "☀︎" },
  { id: "weekly", label: "Weekly", icon: "🗓︎" },
  { id: "monthly", label: "Monthly", icon: "▦" },
  { id: "apps", label: "App Breakdown", icon: "❖" },
  { id: "timeline", label: "Timeline", icon: "⏱︎" },
  { id: "export", label: "Export", icon: "⤻" },
  { id: "settings", label: "Settings", icon: "⛭" },
];

export default function Sidebar() {
  const { view, setView, isTrackingPaused, toggleTracking } = useStore();

  return (
    <aside className="w-[220px] min-w-[220px] flex flex-col bg-wt-surface border-r border-wt-border">
      {/* Logo */}
      <div className="px-5 py-5 border-b border-wt-border">
        <div className="flex items-center gap-3">
          <img
            src={logo}
            alt="WinTrack"
            className="w-8 h-8 rounded-lg"
          />
          <div>
            <div className="text-sm font-semibold text-wt-text">WinTrack</div>
            <div className="text-[10px] text-wt-muted">v2.4.0</div>
          </div>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 px-3 py-4 space-y-0.5 overflow-y-auto">
        {NAV_ITEMS.map((item) => (
          <button
            key={item.id}
            onClick={() => setView(item.id as any)}
            className={clsx(
              "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-all duration-150 text-left",
              view === item.id
                ? "bg-wt-accent/15 text-wt-accent font-medium"
                : "text-wt-muted hover:text-wt-text hover:bg-wt-border/60"
            )}
          >
            <span className="text-base leading-none">{item.icon}</span>
            {item.label}
          </button>
        ))}
      </nav>

      {/* Tracking toggle */}
      <div className="px-3 pb-4">
        <button
          onClick={toggleTracking}
          className={clsx(
            "w-full flex items-center justify-center gap-2 px-3 py-2.5 rounded-lg text-xs font-medium transition-all",
            isTrackingPaused
              ? "bg-wt-green/15 text-wt-green hover:bg-wt-green/25"
              : "bg-wt-amber/15 text-wt-amber hover:bg-wt-amber/25"
          )}
        >
          <span className={clsx(
            "w-1.5 h-1.5 rounded-full",
            isTrackingPaused ? "bg-wt-amber" : "bg-wt-green animate-pulse"
          )} />
          {isTrackingPaused ? "Resume Tracking" : "Pause Tracking"}
        </button>
      </div>
    </aside>
  );
}
