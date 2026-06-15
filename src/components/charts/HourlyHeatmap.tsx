import { useEffect, useState } from "react";
import { api } from "../../utils/api";
import { formatDuration } from "../../utils/helpers";
import { getHeatmapColor } from "../../utils/helpers";

interface Props {
  date: string;
}

export default function HourlyHeatmap({ date }: Props) {
  const [hours, setHours] = useState<number[]>(Array(24).fill(0));

  useEffect(() => {
    api.getHourlyHeatmap(date).then(d => setHours(d.hours));
  }, [date]); 

  const max = Math.max(...hours, 1);

  const getColor = (secs: number) => getHeatmapColor(secs, max);

  const hourLabels = [
    "0","1","2","3","4","5","6","7","8","9","10","11",
    "12","13","14","15","16","17","18","19","20","21","22","23"
  ];

  // Only show labels at these indices to avoid crowding
  const showLabel = new Set([0, 3, 6, 9, 12, 15, 18, 21, 23]);

  return (
    <div className="w-full">
      {/* Bars */}
      <div className="flex gap-0.5 h-28 items-end mb-1">
        {hours.map((secs, i) => (
          <div
            key={i}
            className="flex-1 min-w-0 rounded-t transition-all cursor-default relative group"
            style={{
              height: secs > 0 ? `${Math.max(4, (secs / max) * 100)}%` : "4%",
              backgroundColor: getColor(secs),
            }}
          >
            {/* Tooltip */}
            <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 bg-fp-card border border-fp-border rounded px-2 py-1 text-xs text-fp-text opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-20 pointer-events-none">
              {hourLabels[i]} · {formatDuration(secs)}
            </div>
          </div>
        ))}
      </div>

      {/* Labels - same flex grid as bars, only render text at sparse intervals */}
      <div className="flex gap-0.5">
        {Array.from({ length: 24 }, (_, i) => (
          <div key={i} className="flex-1 min-w-0 text-center overflow-hidden">
            {showLabel.has(i) ? (
              <span className="text-[9px] text-fp-muted leading-none">{hourLabels[i]}</span>
            ) : null}
          </div>
        ))}
      </div>
    </div>
  );
}
