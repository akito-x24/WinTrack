import type { DayStats } from "../../types";
import { formatDuration } from "../../utils/helpers";

interface Props {
  data: DayStats[];
}

export default function WeeklyBarChart({ data }: Props) {
  const today = new Date().toISOString().split("T")[0];
  const max = Math.max(...data.map(d => d.active_seconds), 1);

  const getActiveColor = (date: string) => date === today ? "#60a5fa" : "#3b82f6";
  const getProductiveColor = (date: string) => date === today ? "#34d399" : "#22c55e";

  const dayLabel = (dateStr: string) =>
    new Date(dateStr + "T12:00:00").toLocaleDateString("en-US", { weekday: "short" });

  return (
    <div className="w-full">
      {/* Bars */}
      <div className="flex gap-12 h-36 items-end mb-1">
        {data.map((d) => {
          const activePct  = Math.max(0, (d.active_seconds / max) * 100);
          const prodPct    = Math.max(0, (d.productive_seconds / max) * 100);
          const isToday    = d.date === today;

          return (
            <div key={d.date} className="flex-1 flex gap-0.5 items-end h-full group relative">
              {/* Total bar */}
              <div
                className="flex-1 rounded-t transition-all duration-300"
                style={{
                  height: `${activePct}%`,
                  // backgroundColor: isToday ? "#3b82f6" : "#3b82f6",
                  backgroundColor: getActiveColor(d.date)
                }}
              />
              {/* Productive bar */}
              <div
                className="flex-1 rounded-t transition-all duration-300"
                style={{
                  height: `${prodPct}%`,
                  backgroundColor: getProductiveColor(d.date),
                }}
              />

              {/* Tooltip */}
              <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 bg-fp-card border border-fp-border rounded-lg px-3 py-2 text-xs text-fp-text opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-20 pointer-events-none shadow-lg">
                <p className="font-medium mb-1">{dayLabel(d.date)}{isToday ? " · Today" : ""}</p>
                <p style={{ color: "#1b6cde" }}>Total: {formatDuration(d.active_seconds)}</p>
                <p style={{ color: getProductiveColor(d.date) }}>Productive: {formatDuration(d.productive_seconds)}</p>
              </div>
            </div>
          );
        })}
      </div>

      {/* Day labels — perfectly aligned with bars */}
      <div className="flex gap-1">
        {data.map((d) => (
          <div key={d.date} className="flex-1 text-center">
            <span className={`text-[11px] ${d.date === today ? "text-fp-accent font-medium" : "text-fp-muted"}`}>
              {dayLabel(d.date)}
            </span>
          </div>
        ))}
      </div>

      {/* Legend */}
      <div className="flex items-center gap-4 mt-3 justify-end">
        <div className="flex items-center gap-1.5">
          <span className="w-2.5 h-2.5 rounded-sm bg-[#1b6cde]" />
          <span className="text-[11px] text-fp-muted">Total</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="w-2.5 h-2.5 rounded-sm bg-fp-green" />
          <span className="text-[11px] text-fp-muted">Productive</span>
        </div>
      </div>
    </div>
  );
}
