import { useEffect, useState } from "react";
import { api } from "../utils/api";
import { StatCard, AppRow, SectionHeader, LoadingSpinner } from "../components/ui";
import WeeklyBarChart from "../components/charts/WeeklyBarChart";
import { formatDuration, getWeekStart, subtractDays } from "../utils/helpers";
import type { WeeklyStats, DayStats } from "../types";

// Always return exactly 7 DayStats entries for the week starting at weekStart,
// filling in zeroes for days with no data.
function buildFullWeek(weekStart: string, stats: WeeklyStats | null): DayStats[] {
  return Array.from({ length: 7 }, (_, i) => {
    const d = new Date(weekStart + "T12:00:00");
    d.setDate(d.getDate() + i);
    const dateStr = d.toISOString().split("T")[0];
    const existing = stats?.days.find(x => x.date === dateStr);
    return existing ?? {
      date: dateStr,
      active_seconds: 0,
      idle_seconds: 0,
    };
  });
}

export default function WeeklyAnalytics() {
  const [weekStart, setWeekStart] = useState(getWeekStart());
  const [stats, setStats] = useState<WeeklyStats | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const validDate = weekStart.match(/^\d{4}-\d{2}-\d{2}$/) ? weekStart : getWeekStart();
    setLoading(true);
    api.getWeeklyUsage(validDate)
      .then(setStats)
      .catch(() => setStats(null))
      .finally(() => setLoading(false));
  }, [weekStart]);

  // Always 7 days - empty days get zero values
  const fullWeek = buildFullWeek(weekStart, stats);

  const totalActive     = fullWeek.reduce((s, d) => s + d.active_seconds, 0);
  const activeDays      = fullWeek.filter(d => d.active_seconds > 0).length;
  const avgDaily        = activeDays ? Math.round(totalActive / activeDays) : 0;

  const prevWeek = () => setWeekStart(subtractDays(weekStart, 7));
  const nextWeek = () => {
    const nw = subtractDays(weekStart, -7);
    if (nw <= getWeekStart()) setWeekStart(nw);
  };

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">
      {/* Week nav */}
      <div className="flex items-center justify-between">
        <button onClick={prevWeek} className="fp-btn-ghost px-2">←</button>
        <span className="text-sm font-medium text-fp-text">
          Week of {new Date(weekStart + "T12:00:00").toLocaleDateString("en-US", { month: "long", day: "numeric" })}
        </span>
        <div className="flex items-center gap-2">
          <button
            onClick={nextWeek}
            disabled={weekStart >= getWeekStart()}
            className="fp-btn-ghost px-2 disabled:opacity-40"
          >→</button>
          <button onClick={() => setWeekStart(getWeekStart())} className="fp-btn-ghost text-xs">
            This Week
          </button>
        </div>
      </div>

      {loading ? (
        <div className="fp-card flex justify-center py-12"><LoadingSpinner /></div>
      ) : (
        <>
          {/* Summary cards */}
          <div className="grid grid-cols-3 gap-4">

            <StatCard label="Total Active"  value={formatDuration(totalActive)}  accent="#3b82f6"/>
            <StatCard label="Apps Used" value={String(stats?.top_apps?.length ?? 0)} sub="Unique tracked apps" accent="#22c55e"/>
            <StatCard label="Daily Avg" value={formatDuration(avgDaily)} accent="#8b5cf6" 
            sub={activeDays ? `across ${activeDays} active day${activeDays !== 1 ? "s" : ""}`: "no activity yet"  }/>
          </div>

          {/* Chart - always shows all 7 days */}
          <div className="fp-card">
            <SectionHeader title="Daily Activity" />
            <WeeklyBarChart data={fullWeek} />
          </div>

          {/* Top apps - only show if there's actual data */}
          {stats?.top_apps && stats.top_apps.length > 0 && (
            <div className="fp-card">
              <SectionHeader title="Top Apps This Week" />
              <div className="space-y-2">
                {stats.top_apps.slice(0, 6).map((app, i) => (
                  <AppRow
                    key={app.app_name}
                    name={app.app_name}
                    exePath={app.executable_path}
                    category={app.category}
                    durationSeconds={app.duration_seconds}
                    maxSeconds={Math.max(...stats.top_apps.map(x => x.duration_seconds), 1)}
                    rank={i + 1}
                  />
                ))}
              </div>
            </div>
          )}

          {/* Empty state - only shown when zero activity all week */}
          {totalActive === 0 && (
            <div className="fp-card flex flex-col items-center justify-center py-10 text-center">
              <p className="text-fp-muted text-sm">No activity recorded this week yet</p>
            </div>
          )}
        </>
      )}
    </div>
  );
}
