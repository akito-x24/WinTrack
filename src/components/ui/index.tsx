import clsx from "clsx";
import { formatDuration, getAppIcon, getAppInitials, percentOf } from "../../utils/helpers";
import type { AppCategory } from "../../types";
import { CATEGORY_COLORS } from "../../types";

// ─── StatCard ─────────────────────────────────────────────────────────────────

interface StatCardProps {
  label: string;
  value: string;
  sub?: string;
  accent?: string;
  icon?: React.ReactNode;
}

export function StatCard({ label, value, sub, accent, icon }: StatCardProps) {
  return (
    <div className="fp-card flex flex-col gap-2 relative overflow-hidden">
      {accent && (
        <div
          className="absolute top-0 left-0 right-0 h-0.5"
          style={{ background: accent }}
        />
      )}
      <div className="flex items-start justify-between">
        <span className="fp-label">{label}</span>
        {icon && <span className="text-fp-muted">{icon}</span>}
      </div>
      <div className="fp-value">{value}</div>
      {sub && <div className="text-xs text-fp-muted">{sub}</div>}
    </div>
  );
}

// ─── ProductivityRing ─────────────────────────────────────────────────────────

interface ProductivityRingProps {
  score: number;
  size?: number;
}

export function ProductivityRing({ score, size = 80 }: ProductivityRingProps) {
  const r = (size - 8) / 2;
  const circ = 2 * Math.PI * r;
  const offset = circ - (score / 100) * circ;
  const color = score >= 70 ? "#22c55e" : score >= 40 ? "#f59e0b" : "#ef4444";

  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
      <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="#252a36" strokeWidth={6} />
      <circle
        cx={size / 2}
        cy={size / 2}
        r={r}
        fill="none"
        stroke={color}
        strokeWidth={6}
        strokeDasharray={circ}
        strokeDashoffset={offset}
        strokeLinecap="round"
        transform={`rotate(-90 ${size / 2} ${size / 2})`}
        style={{ transition: "stroke-dashoffset 0.6s ease" }}
      />
      <text
        x={size / 2}
        y={size / 2 + 1}
        textAnchor="middle"
        dominantBaseline="middle"
        fill={color}
        fontSize="14"
        fontWeight="600"
        fontFamily="DM Sans"
      >
        {score}%
      </text>
    </svg>
  );
}

// ─── AppRow ───────────────────────────────────────────────────────────────────

interface AppRowProps {
  name: string;
  exePath: string;
  category: AppCategory;
  durationSeconds: number;
  maxSeconds: number;
  rank?: number;
}

export function AppRow({ name, exePath, category, durationSeconds, maxSeconds, rank }: AppRowProps) {
  const pct = percentOf(durationSeconds, maxSeconds);
  const color = CATEGORY_COLORS[category] || "#64748b";
  const initials = getAppInitials(name);
  const bgColor = getAppIcon(name);

  return (
    <div className="flex items-center gap-3 py-2.5 group">
      {rank && (
        <span className="text-xs text-fp-muted w-4 text-right shrink-0">{rank}</span>
      )}
      {/* Icon */}
      <div
        className="w-8 h-8 rounded-lg flex items-center justify-center text-white text-xs font-bold shrink-0"
        style={{ background: bgColor + "33", color: bgColor, border: `1px solid ${bgColor}33` }}
      >
        {initials || "?"}
      </div>
      {/* Name + bar */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between mb-1">
          <span className="text-sm text-fp-text truncate">{name.replace(/\.exe$/i, "")}</span>
          <span className="text-xs text-fp-muted ml-2 shrink-0">{formatDuration(durationSeconds)}</span>
        </div>
        <div className="h-1 bg-fp-border rounded-full overflow-hidden">
          <div
            className="h-full rounded-full transition-all duration-500"
            style={{ width: `${pct}%`, background: color }}
          />
        </div>
      </div>
      {/* Category */}
      <CategoryBadge category={category} />
    </div>
  );
}

// ─── CategoryBadge ────────────────────────────────────────────────────────────

interface CategoryBadgeProps {
  category: AppCategory;
  size?: "sm" | "xs";
}

export function CategoryBadge({ category, size = "xs" }: CategoryBadgeProps) {
  const color = CATEGORY_COLORS[category] || "#64748b";
  return (
    <span
      className={clsx(
        "rounded-full font-medium shrink-0",
        size === "xs" ? "text-[10px] px-2 py-0.5" : "text-xs px-2.5 py-1"
      )}
      style={{ background: color + "22", color }}
    >
      {category}
    </span>
  );
}

// ─── SectionHeader ────────────────────────────────────────────────────────────

export function SectionHeader({ title, action }: { title: string; action?: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between mb-4">
      <h2 className="text-sm font-semibold text-fp-text">{title}</h2>
      {action}
    </div>
  );
}

// ─── EmptyState ──────────────────────────────────────────────────────────────

export function EmptyState({ message = "No data available yet" }: { message?: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-fp-muted gap-3">
      <div className="w-12 h-12 rounded-xl bg-fp-border flex items-center justify-center text-xl">◌</div>
      <p className="text-sm">{message}</p>
    </div>
  );
}

// ─── LoadingSpinner ───────────────────────────────────────────────────────────

export function LoadingSpinner() {
  return (
    <div className="flex items-center justify-center py-16">
      <div className="w-6 h-6 border-2 border-fp-border border-t-fp-accent rounded-full animate-spin" />
    </div>
  );
}

// ─── TimeBar ─────────────────────────────────────────────────────────────────

interface TimeBarProps {
  label: string;
  seconds: number;
  maxSeconds: number;
  color?: string;
}

export function TimeBar({ label, seconds, maxSeconds, color = "#3b82f6" }: TimeBarProps) {
  const pct = percentOf(seconds, maxSeconds);
  return (
    <div className="flex items-center gap-3">
      <span className="text-xs text-fp-muted w-20 shrink-0 truncate">{label}</span>
      <div className="flex-1 h-2 bg-fp-border rounded-full overflow-hidden">
        <div
          className="h-full rounded-full transition-all duration-500"
          style={{ width: `${pct}%`, background: color }}
        />
      </div>
      <span className="text-xs text-fp-muted w-12 text-right shrink-0">{formatDuration(seconds)}</span>
    </div>
  );
}
