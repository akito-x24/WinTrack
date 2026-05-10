import { useState } from "react";
import { useStore } from "../store";
import { api } from "../utils/api";
import {
  StatCard, AppRow, SectionHeader, LoadingSpinner, ProductivityRing
} from "../components/ui";
import CategoryPieChart from "../components/charts/CategoryPieChart";
import HourlyHeatmap from "../components/charts/HourlyHeatmap";
import { formatDuration, todayString, subtractDays } from "../utils/helpers";
import type { DailyStats } from "../types";

export default function DailyAnalytics() {
  const [date, setDate] = useState(todayString());
  const [stats, setStats] = useState<DailyStats | null>(useStore.getState().todayStats);
  const [loading, setLoading] = useState(false);

  const fetchDate = async (d: string) => {
    setLoading(true);
    try {
      const data = await api.getDailyUsage(d);
      setStats(data);
    } finally {
      setLoading(false);
    }
  };

  const changeDate = (d: string) => {
    setDate(d);
    fetchDate(d);
  };

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">
      {/* Date picker */}
      <div className="flex items-center gap-3">
        <button
          onClick={() => changeDate(subtractDays(date, 1))}
          className="fp-btn-ghost px-2 py-1.5"
        >
          ←
        </button>
        <input
          type="date"
          value={date}
          max={todayString()}
          onChange={e => changeDate(e.target.value)}
          className="bg-fp-card border border-fp-border text-fp-text text-sm rounded-lg px-3 py-1.5 focus:outline-none focus:border-fp-accent"
        />
        <button
          onClick={() => {
            if (date < todayString()) changeDate(subtractDays(date, -1));
          }}
          className="fp-btn-ghost px-2 py-1.5"
          disabled={date >= todayString()}
        >
          →
        </button>
        <button onClick={() => changeDate(todayString())} className="fp-btn-ghost text-xs">
          Today
        </button>
      </div>

      {loading ? <LoadingSpinner /> : stats ? (
        <>
          {/* Stats row */}
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <StatCard label="Total Active" value={formatDuration(stats.total_active_seconds)} accent="#3b82f6" />
            <StatCard label="Productive" value={formatDuration(stats.productive_seconds)} accent="#22c55e" />
            <StatCard label="Idle" value={formatDuration(stats.total_idle_seconds)} accent="#f59e0b" />
            <div className="fp-card flex items-center gap-4">
              <ProductivityRing score={stats.productivity_score} size={64} />
              <div>
                <div className="fp-label">Score</div>
                <div className="text-lg font-bold text-fp-text">{stats.productivity_score}%</div>
              </div>
            </div>
          </div>

          {/* Heatmap */}
          <div className="fp-card">
            <SectionHeader title="Hourly Breakdown" />
            <HourlyHeatmap date={date} />
          </div>

          {/* App list + categories */}
          <div className="grid grid-cols-3 gap-4">
            <div className="col-span-2 fp-card">
              <SectionHeader title="Apps Used" />
              {stats.apps.length > 0 ? (
                <div className="divide-y divide-fp-border/50">
                  {stats.apps.map((app, i) => (
                    <AppRow
                      key={app.executable_path}
                      name={app.app_name}
                      exePath={app.executable_path}
                      category={app.category}
                      durationSeconds={app.duration_seconds}
                      maxSeconds={stats.apps[0].duration_seconds}
                      rank={i + 1}
                    />
                  ))}
                </div>
              ) : (
                <p className="text-sm text-fp-muted text-center py-8">No data for this day</p>
              )}
            </div>
            <div className="fp-card">
              <SectionHeader title="Categories" />
              <CategoryPieChart data={stats.categories} />
            </div>
          </div>
        </>
      ) : (
        <p className="text-fp-muted text-sm">No data for {date}</p>
      )}
    </div>
  );
}
