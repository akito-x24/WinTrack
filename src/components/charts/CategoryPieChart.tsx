import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from "recharts";
import { CATEGORY_COLORS } from "../../types";
import { formatDuration } from "../../utils/helpers";
import type { CategoryUsage } from "../../types";

interface Props {
  data: CategoryUsage[];
}

export default function CategoryPieChart({ data }: Props) {
  if (!data || data.length === 0) {
    return <div className="flex items-center justify-center h-48 text-fp-muted text-sm">No data</div>;
  }

  const chartData = data.map(d => ({
    name: d.category,
    value: d.duration_seconds,
    color: CATEGORY_COLORS[d.category as keyof typeof CATEGORY_COLORS] || "#64748b",
  }));

  const CustomTooltip = ({ active, payload }: any) => {
    if (!active || !payload?.length) return null;
    return (
      <div className="bg-fp-card border border-fp-border rounded-lg px-3 py-2">
        <p className="text-xs font-medium text-fp-text">{payload[0].name}</p>
        <p className="text-xs text-fp-muted">{formatDuration(payload[0].value)}</p>
      </div>
    );
  };

  const renderLegend = (props: any) => {
    const { payload } = props;
    return (
      <ul className="flex flex-wrap gap-2 justify-center mt-3">
        {payload.map((entry: any, i: number) => (
          <li key={i} className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full" style={{ background: entry.color }} />
            <span className="text-[11px] text-fp-muted">{entry.value}</span>
          </li>
        ))}
      </ul>
    );
  };

  return (
    <ResponsiveContainer width="100%" height={220}>
      <PieChart>
        <Pie
          data={chartData}
          innerRadius={60}
          outerRadius={85}
          paddingAngle={3}
          dataKey="value"
          strokeWidth={0}
        >
          {chartData.map((entry, i) => (
            <Cell key={i} fill={entry.color} />
          ))}
        </Pie>
        <Tooltip content={<CustomTooltip />} />
        <Legend content={renderLegend} />
      </PieChart>
    </ResponsiveContainer>
  );
}
