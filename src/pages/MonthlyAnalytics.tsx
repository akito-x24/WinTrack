import { useEffect, useState } from "react";
import { api } from "../utils/api";
import { SectionHeader, LoadingSpinner, StatCard } from "../components/ui";
import type { MonthlyStats } from "../types";

export default function MonthlyAnalytics() {
  const now = new Date();
  const [year, setYear] = useState(now.getFullYear());
  const [month, setMonth] = useState(now.getMonth() + 1);
  const [stats, setStats] = useState<MonthlyStats | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    api.getMonthlyUsage(year, month).then(setStats).finally(() => setLoading(false));
  }, [year, month]);

  const prevMonth = () => {
    if (month === 1) { setYear(y => y - 1); setMonth(12); }
    else setMonth(m => m - 1);
  };
  const nextMonth = () => {
    const isCurrentMonth = year === now.getFullYear() && month === now.getMonth() + 1;
    if (isCurrentMonth) return;
    if (month === 12) { setYear(y => y + 1); setMonth(1); }
    else setMonth(m => m + 1);
  };

  const totalActive = stats?.days.reduce((s, d) => s + d.active_seconds, 0) ?? 0;
  const maxDay = Math.max(...(stats?.days.map(d => d.active_seconds) ?? [1]));
  const avgDaily = stats?.days.length ? Math.round(totalActive / stats.days.length) : 0;

  const MONTH_NAMES = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">
      {/* Month nav */}
      <div className="flex items-center gap-3">
        <button onClick={prevMonth} className="fp-btn-ghost px-2">←</button>
        <span className="text-sm text-fp-text font-medium">{MONTH_NAMES[month - 1]} {year}</span>
        <button onClick={nextMonth} className="fp-btn-ghost px-2">→</button>
      </div>

      {loading ? <LoadingSpinner /> : stats ? (
        <>
          <div className="grid grid-cols-3 gap-4">
            <StatCard label="Total Active" value={formatDuration(totalActive)} accent="#3b82f6" />
            <StatCard label="Daily Average" value={formatDuration(avgDaily)} accent="#22c55e" />
            <StatCard label="Active Days" value={`${stats.days.filter(d => d.active_seconds > 0).length}`} accent="#8b5cf6" />
          </div>

          {/* Calendar heatmap */}
          <div className="fp-card">
            <SectionHeader title="Monthly Calendar" />
            <CalendarHeatmap days={stats.days} year={year} month={month} maxDay={maxDay} />
          </div>
        </>
      ) : null}
    </div>
  );
}

function CalendarHeatmap({
  days, year, month, maxDay
}: {
  days: { date: string; active_seconds: number }[];
  year: number; month: number; maxDay: number;
}) {
  const dayMap = Object.fromEntries(days.map(d => [d.date, d.active_seconds]));

  const firstDay = new Date(year, month - 1, 1).getDay();
  const daysInMonth = new Date(year, month, 0).getDate();
  const blanks = Array(firstDay === 0 ? 6 : firstDay - 1).fill(null);
  const allDays = [...blanks, ...Array.from({ length: daysInMonth }, (_, i) => i + 1)];

  const getColor = (secs: number) => {
    if (!secs) return "#1a1e28";
    const pct = secs / maxDay;
    if (pct < 0.25) return "#1e3a5f";
    if (pct < 0.5) return "#1d4ed8";
    if (pct < 0.75) return "#2563eb";
    return "#3b82f6";
  };

  const DOW = ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"];

  return (
    <div>
      <div className="grid grid-cols-7 gap-1 mb-2">
        {DOW.map(d => (
          <div key={d} className="text-[10px] text-fp-muted text-center">{d}</div>
        ))}
      </div>
      <div className="grid grid-cols-7 gap-1">
        {allDays.map((day, i) => {
          if (day === null) return <div key={`b-${i}`} />;
          const dateStr = `${year}-${String(month).padStart(2,"0")}-${String(day).padStart(2,"0")}`;
          const secs = dayMap[dateStr] ?? 0;
          return (
            <div
              key={dateStr}
              className="aspect-square rounded flex items-center justify-center group relative cursor-default"
              style={{ background: getColor(secs) }}
            >
              <span className="text-[10px] text-white/50">{day}</span>
              {secs > 0 && (
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 bg-fp-card border border-fp-border rounded px-2 py-1 text-[10px] whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity z-10 pointer-events-none">
                  {formatDuration(secs)}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function formatDuration(s: number) {
  if (s < 60) return `${s}s`;
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}
