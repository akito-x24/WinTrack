import { useStore } from "../store";
import {
  StatCard,
  AppRow,
  SectionHeader,
  EmptyState,
  LoadingSpinner,
} from "../components/ui";
import CategoryPieChart from "../components/charts/CategoryPieChart";
import WeeklyBarChart from "../components/charts/WeeklyBarChart";
import HourlyHeatmap from "../components/charts/HourlyHeatmap";
import { formatDuration, todayString } from "../utils/helpers";
import type { MonthlyStats } from "../types";

export default function Dashboard() {
  const { todayStats, weeklyStats, loading } = useStore();

  if (loading.today && !todayStats) return <LoadingSpinner />;

  const stats = todayStats;
  const today = todayString();

  return (
    <div className="space-y-6 max-w-7xl animate-slide-up">
      {/* Top stat cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Total Active Time"
          value={formatDuration(stats?.total_active_seconds ?? 0)}
          sub={`${Math.round((stats?.total_active_seconds ?? 0) / 3600 * 10) / 10}h today`}
          accent="#3b82f6"
        />

        <StatCard
          label="Daily Average"
          value="Coming Soon"
          sub="30-day average"
          accent="#22c55e"
        />

        <StatCard
          label="Apps Used"
          value={String(stats?.apps?.length ?? 0)}
          sub="Tracked today"
          accent="#8b5cf6"
        />

        <StatCard
          label="Top App"
          value={stats?.apps?.[0]?.app_name ?? "None"}
          sub={
            stats?.apps?.[0]
              ? formatDuration(stats.apps[0].duration_seconds)
              : "No usage"
          }
          accent="#06b6d4"
        />
      </div>

      {/* Main content grid */}
      <div className="grid grid-cols-3 gap-4">
        <div className="col-span-2 fp-card">
          <SectionHeader title="Top Apps Today" />
          {stats?.apps && stats.apps.length > 0 ? (
            <div className="divide-y divide-fp-border/50">
              {stats.apps.slice(0, 8).map((app, i) => (
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
            <EmptyState message="No app usage recorded today yet" />
          )}
        </div>

        <div className="fp-card">
          <SectionHeader title="By Category" />
          {stats?.categories ? (
            <CategoryPieChart categories={stats.categories} apps={stats.apps}/>
          ) : (
            <EmptyState />
          )}
        </div>
      </div>

      {/* Hourly heatmap */}
      <div className="fp-card">
        <SectionHeader title="Hourly Activity" />
        <HourlyHeatmap date={today} />
      </div>

      {/* Weekly trend */}
      <div className="fp-card">
        <SectionHeader title="This Week" />
        <WeeklyBarChart
          data={Array.from({ length: 7 }, (_, i) => {
            const d = new Date();
            d.setDate(
              d.getDate() -
                d.getDay() +
                (d.getDay() === 0 ? -6 : 1) +
                i
            );

            const dateStr = d.toISOString().split("T")[0];
            const existing = weeklyStats?.days.find(
              (x) => x.date === dateStr
            );

            return (
              existing ?? {
                date: dateStr,
                active_seconds: 0,
                idle_seconds: 0,
              }
            );
          })}
        />
      </div>
    </div>
  );
}