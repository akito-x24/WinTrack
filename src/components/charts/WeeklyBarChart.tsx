import type { DayStats } from "../../types";
import { formatDuration } from "../../utils/helpers";

interface Props {
  data: DayStats[];
}

export default function WeeklyBarChart({ data }: Props) {
  // const today = new Date().toISOString().split("T")[0];
  const today = new Date().toLocaleDateString("en-CA");
  const max = Math.max(...data.map(d => d.active_seconds), 1);
  const dayLabel = (dateStr: string) =>
    new Date(dateStr + "T12:00:00").toLocaleDateString("en-US", { weekday: "short" });

  return (
    <div className="w-full">
      {/* Bars */}
      <div className="flex gap-2 h-36 items-end mb-1">
        {data.map((d) => {
          const activePct = Math.max(0, (d.active_seconds / max) * 100);
          const isToday = d.date === today;

          return (
            <div key={d.date} className="flex-1 flex gap-0.5 items-end h-full group relative">
              {/* Total bar */}
              <div
                className="flex-1 rounded-t transition-all duration-300"
                style={{
                  height: `${activePct}%`,
                  backgroundColor: isToday ? "#092ee7" : "#3581fb",
                }}
              />

              {/* Tooltip */}
              <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 bg-fp-card border border-fp-border rounded-lg px-3 py-2 text-xs text-fp-text opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-20 pointer-events-none shadow-lg">
                <p className="font-medium mb-1">{dayLabel(d.date)}{isToday ? " · Today" : ""}</p>
                <p style={{ color: "#0f5cc8" }}>Total: {formatDuration(d.active_seconds)}</p>
              </div>
            </div>
          );
        })}
      </div>

      {/* Day labels - perfectly aligned with bars */}
      <div className="flex gap-1">
        {data.map((d) => (
          <div key={d.date} className="flex-1 text-center">
            <span className={`text-[11px] ${d.date === today ? "text-fp-accent font-medium" : "text-fp-muted"}`}>
              {dayLabel(d.date)}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}