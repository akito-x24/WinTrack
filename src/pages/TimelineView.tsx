import { useEffect, useState } from "react";
import { api } from "../utils/api";
import { AppIcon, CategoryBadge, LoadingSpinner } from "../components/ui";
import { todayString, subtractDays, formatTime, formatDuration } from "../utils/helpers";
import { CATEGORY_COLORS } from "../types";
import type { Timeline, UsageSession } from "../types";

export default function TimelineView() {
  const [date, setDate] = useState(todayString());
  const [timeline, setTimeline] = useState<Timeline | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    api.getTimeline(date).then(setTimeline).finally(() => setLoading(false));
  }, [date]);

  const changeDate = (d: string) => setDate(d);

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">
      {/* Date nav */}
      <div className="flex items-center gap-3">
        <button onClick={() => changeDate(subtractDays(date, 1))} className="wt-btn-ghost px-2">←</button>
        <input
          type="date"
          value={date}
          max={todayString()}
          onChange={e => changeDate(e.target.value)}
          className="bg-wt-card border border-wt-border text-wt-text text-sm rounded-lg px-3 py-1.5 focus:outline-none focus:border-wt-accent"
        />
        <button
          onClick={() => { if (date < todayString()) changeDate(subtractDays(date, -1)); }}
          disabled={date >= todayString()}
          className="wt-btn-ghost px-2"
        >→</button>
        <button onClick={() => changeDate(todayString())} className="wt-btn-ghost text-xs">Today</button>
      </div>

      {loading ? <LoadingSpinner /> : timeline?.sessions.length ? (
        <div className="wt-card p-0 overflow-hidden">
          <div className="px-5 py-4 border-b border-wt-border wt-label">
            {timeline.sessions.length} sessions
          </div>
          <div className="divide-y divide-wt-border/40 max-h-[600px] overflow-y-auto">
            {timeline.sessions.map((s, i) => (
              <SessionRow key={i} session={s} />
            ))}
          </div>
        </div>
      ) : (
        <div className="wt-card text-center py-12 text-wt-muted text-sm">
          No sessions recorded for {date}
        </div>
      )}
    </div>
  );
}

function SessionRow({ session }: { session: UsageSession }) {
  const color = CATEGORY_COLORS[session.category] || "#64748b";

  return (
    <div className={`flex items-start gap-4 px-5 py-3 ${session.was_idle ? "opacity-40" : ""}`}>
      {/* Time */}
      <div className="text-xs font-mono text-wt-muted w-12 shrink-0 mt-0.5">
        {formatTime(session.start_time)}
      </div>
      {/* Color bar */}
      <div className="w-1 self-stretch rounded-full shrink-0" style={{ background: color }} />
      <AppIcon
        name={session.app_name}
        iconData={session.icon_data}
        className="w-8 h-8"
      />
      {/* Info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5">
          <span className="text-sm font-medium text-wt-text">
            {session.app_name.replace(/\.exe$/i, "")}
          </span>
          {session.was_idle && (
            <span className="text-[10px] text-wt-muted bg-wt-border px-1.5 py-0.5 rounded">idle</span>
          )}
          <CategoryBadge category={session.category} />
        </div>
        {session.window_title && (
          <div className="text-xs text-wt-muted truncate">{session.window_title}</div>
        )}
      </div>
      {/* Duration */}
      <div className="text-xs text-wt-muted shrink-0 mt-0.5">
        {formatDuration(session.duration_seconds)}
      </div>
    </div>
  );
}
