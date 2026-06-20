import {
  PieChart,
  Pie,
  Cell,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

import { CATEGORY_COLORS } from "../../types";
import { AppIcon } from "../ui";
import { formatDuration } from "../../utils/helpers";

import type {
  CategoryUsage,
  AppUsage,
} from "../../types";

interface Props {
  categories: CategoryUsage[];
  apps: AppUsage[];
}

const CATEGORY_SHADES: Record<string, string[]> = {
  Development: [
    "#3385FF",
    "#66A3FF",
    "#99C2FF",
    "#CCE0FF",
  ],

  Productive: [
    "#FFC233",
    "#FFD466",
    "#FFE699",
    "#FFF2CC",
  ],

  Study: [
    "#A78BFA",
    "#C4B5FD",
    "#DDD6FE",
    "#EDE9FE",
  ],

  Social: [
    "#FF66BF",
    "#FF99D6",
    "#FFC2E8",
    "#FFE0F3",
  ],

  Entertainment: [
    "#4DEEFF",
    "#80F3FF",
    "#B3F8FF",
    "#D9FCFF",
  ],

  Gaming: [
    "#FF7333",
    "#FF9966",
    "#FFBF99",
    "#FFE0CC",
  ],

  Tools: [
    "#2DD4BF",
    "#5EEAD4",
    "#99F6E4",
    "#CCFBF1",
  ],

  Other: [
    "#B1BDCC",
    "#CBD5E1",
    "#E2E8F0",
    "#F1F5F9",
  ],
};

export default function CategoryPieChart({
  categories,
  apps,
}: Props) {
  if (!categories || categories.length === 0) {
    return (
      <div className="flex items-center justify-center h-48 text-wt-muted text-sm">
        No data
      </div>
    );
  }

  const categoryData = categories.map((d) => ({
    name: d.category,
    value: d.duration_seconds,
    color:
      CATEGORY_COLORS[
        d.category as keyof typeof CATEGORY_COLORS
      ] || "#64748b",
  }));

  const appData: {
    name: string;
    value: number;
    color: string;
    iconData?: string | null;
  }[] = [];

  for (const category of categories) {
    const categoryApps = apps
      .filter((app) => app.category === category.category)
      .sort(
        (a, b) =>
          b.duration_seconds - a.duration_seconds
      );

    categoryApps.forEach((app, index) => {
      const shades =
        CATEGORY_SHADES[app.category] ??
        CATEGORY_SHADES.Other;

      appData.push({
        name: app.app_name,
        value: app.duration_seconds,
        color: shades[index % shades.length],
        iconData: app.icon_data,
      });
    });
  }

  const CustomTooltip = ({
    active,
    payload,
  }: any) => {
    if (!active || !payload?.length) return null;

    return (
      <div className="bg-wt-card border border-wt-border rounded-lg px-3 py-2">
        <div className="flex items-center gap-2">
          {payload[0].payload.iconData && (
            <AppIcon
              name={payload[0].name}
              iconData={payload[0].payload.iconData}
              className="w-5 h-5"
            />
          )}
          <p className="text-xs font-medium text-wt-text">
            {payload[0].name}
          </p>
        </div>
        <p className="text-xs text-wt-muted">
          {formatDuration(payload[0].value)}
        </p>
      </div>
    );
  };

  return (
    <ResponsiveContainer width="100%" height={300}>
      <PieChart>
        {/* Inner Ring - Categories */}
        <Pie
          data={categoryData}
          innerRadius={30}
          outerRadius={75}
          dataKey="value"
          stroke="#1a1e28"
          strokeWidth={0.75}
        >
          {categoryData.map((entry, i) => (
            <Cell
              key={i}
              fill={entry.color}
            />
          ))}
        </Pie>

        {/* Outer Ring - Apps */}
        <Pie
          data={appData}
          innerRadius={75}
          outerRadius={120}
          dataKey="value"
          stroke="#1a1e28"
          strokeWidth={0.5}
        >
          {appData.map((entry, i) => (
            <Cell
              key={i}
              fill={entry.color}
            />
          ))}
        </Pie>

        <Tooltip content={<CustomTooltip />} />
      </PieChart>
    </ResponsiveContainer>
  );
}
