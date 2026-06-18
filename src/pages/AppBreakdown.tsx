import { useState, useEffect } from "react";
import { useStore } from "../store";
import { AppIcon, CategoryBadge, LoadingSpinner, EmptyState } from "../components/ui";
import { formatDuration } from "../utils/helpers";
import { CATEGORY_LABELS, CATEGORY_COLORS } from "../types";
import type { App, AppCategory } from "../types";
import clsx from "clsx";

type EditMode = "category" | "name" | null;

export default function AppBreakdown() {
  const {
    appList,
    fetchAppList,
    updateAppCategory,
    updateAppDisplayName,
    updateAppDailyLimit,
    setAppIgnored,
    loading,
  } = useStore();
  const [search, setSearch] = useState("");
  const [catFilter, setCatFilter] = useState<AppCategory | "All">("All");
  const [showIgnored, setShowIgnored] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editMode, setEditMode] = useState<EditMode>(null);

  useEffect(() => { fetchAppList(); }, []);

  const filtered = (appList ?? [])
    .filter(app => {
      if (!showIgnored && app.is_ignored) return false;

      const matchSearch =
        app.display_name.toLowerCase().includes(search.toLowerCase()) ||
        app.app_name.toLowerCase().includes(search.toLowerCase());

      const matchCat =
        catFilter === "All" || app.category === catFilter;

      return matchSearch && matchCat;
    })
    .sort((a, b) => {
      const getPriority = (app: App) => {
        if (app.is_ignored) return 0;

        if (
          app.daily_limit_minutes &&
          app.today_seconds
        ) {
          const pct =
            (app.today_seconds / (app.daily_limit_minutes * 60)) * 100;

          if (pct >= 100) return 3;
          if (pct >= 90) return 2;
        }

        return 1;
      };

      const priorityDiff =
        getPriority(b) - getPriority(a);

      if (priorityDiff !== 0) {
        return priorityDiff;
      }

      return (b.today_seconds ?? 0) - (a.today_seconds ?? 0);
    });

  const openEdit = (id: number, mode: EditMode) => {
    setEditingId(id);
    setEditMode(mode);
  };
  const closeEdit = () => { setEditingId(null); setEditMode(null); };

  return (
    <div className="space-y-6">
      {/* Filters */}
      <div className="flex flex-wrap items-center gap-3">
        <input
          type="text"
          placeholder="Search apps..."
          value={search}
          onChange={e => setSearch(e.target.value)}
          className="bg-fp-card border border-fp-border text-fp-text text-sm rounded-lg px-3 py-2 w-52 focus:outline-none focus:border-fp-accent placeholder:text-fp-muted"
        />
        <div className="flex flex-wrap gap-2">
          {(["All", ...CATEGORY_LABELS] as const).map(cat => (
            <button
              key={cat}
              onClick={() => setCatFilter(cat as AppCategory | "All")}
              className={`text-xs px-3 py-1.5 rounded-full transition-all ${catFilter === cat ? "text-white font-medium"
                : "text-fp-muted hover:text-fp-text bg-fp-card border border-fp-border"
                }`}
              style={catFilter === cat ? {
                background: cat === "All" ? "#3b82f6" : CATEGORY_COLORS[cat as AppCategory],
              } : {}}
            >
              {cat}
            </button>
          ))}
        </div>
        <button
          onClick={() => setShowIgnored(v => !v)}
          className={clsx(
            "ml-auto text-xs px-3 py-1.5 rounded-lg border transition-all",
            showIgnored
              ? "bg-fp-amber/15 text-fp-amber border-fp-amber/30"
              : "bg-fp-card text-fp-muted border-fp-border hover:text-fp-text"
          )}
        >
          {showIgnored ? "Hide ignored" : "Show ignored"}
        </button>
      </div>

      {/* App table */}
      <div className="fp-card p-0 overflow-hidden">
        <div className="px-5 py-3 border-b border-fp-border bg-fp-surface/50 flex items-center justify-between">
          <span className="text-sm font-medium text-fp-text">{filtered.length} apps</span>
          {/* <span className="text-xs text-fp-muted">Click ✎ to rename · Click category to change · Click eye to ignore</span> */}
        </div>
        {loading.apps && !appList ? (
          <div className="p-12 flex justify-center"><LoadingSpinner /></div>
        ) : filtered.length === 0 ? (
          <div className="p-12"><EmptyState message="No apps match your filters" /></div>
        ) : (
          <div className="divide-y divide-fp-border/50 max-h-[600px] overflow-y-auto">
            {/* <table className="w-full">
              <tbody> */}
            <table className="w-full">
              <thead className="sticky top-0 bg-fp-surface border-b border-fp-border">
                <tr className="text-xs text-fp-muted">
                  <th className="px-5 py-3 text-left">App</th>
                  <th className="px-4 py-3 text-right">Today</th>
                  <th className="px-4 py-3 text-right">Total</th>
                  <th className="px-4 py-3 text-right">Category</th>
                  <th className="px-4 py-3 text-right">Limit</th>
                  <th className="px-4 py-3 text-right">Reminder</th>
                  <th className="px-4 py-3 text-right">Soft Lock</th>
                  <th className="px-4 py-3 text-right">Status</th>
                  <th className="w-20 px-5 py-3 text-center">Ignore</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map(app => (
                  <AppEditRow
                    key={app.id}
                    app={app}
                    editMode={editingId === app.id ? editMode : null}
                    onOpenEdit={(mode) => openEdit(app.id, mode)}
                    onClose={closeEdit}
                    onSaveName={(name) => { updateAppDisplayName(app.id, name); closeEdit(); }}
                    onSaveCat={(cat) => { updateAppCategory(app.id, cat); closeEdit(); }}
                    onToggleIgnore={() => setAppIgnored(app.id, !app.is_ignored)}
                  />
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}

function AppEditRow({ app, editMode, onOpenEdit, onClose, onSaveName, onSaveCat, onToggleIgnore }: {
  app: App;
  editMode: EditMode;
  onOpenEdit: (mode: EditMode) => void;
  onClose: () => void;
  onSaveName: (name: string) => void;
  onSaveCat: (cat: AppCategory) => void;
  onToggleIgnore: () => void;
}) {
  const [nameInput, setNameInput] = useState(app.display_name);
  const [editingLimit, setEditingLimit] = useState(false);
  const [limitInput, setLimitInput] = useState(app.daily_limit_minutes?.toString() ?? "");
  const [editingReminder, setEditingReminder] = useState(false);
  const [editingSoftLock, setEditingSoftLock] = useState(false);

  useEffect(() => { setNameInput(app.display_name); }, [app.display_name]);

  return (
    <tr className={clsx(
      "group hover:bg-fp-surface/50 transition-colors",
      app.is_ignored && "opacity-50"
    )}>
      <td className="px-5 py-3">
        <div className="flex items-center gap-4">

          {/* Icon */}
          <AppIcon
            name={app.display_name || app.app_name}
            iconData={app.icon_data}
            className="w-9 h-9"
          />

          {/* Name + path */}
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              {editMode === "name" ? (
                <div className="flex items-center gap-2">
                  <input
                    autoFocus
                    value={nameInput}
                    onChange={e => setNameInput(e.target.value)}
                    onKeyDown={e => {
                      if (e.key === "Enter" && nameInput.trim()) onSaveName(nameInput.trim());
                      if (e.key === "Escape") onClose();
                    }}
                    className="bg-fp-surface border border-fp-accent text-fp-text text-sm rounded px-2 py-0.5 w-48 focus:outline-none"
                  />
                  <button
                    onClick={() => { if (nameInput.trim()) onSaveName(nameInput.trim()); }}
                    disabled={!nameInput.trim()}
                    className="text-xs text-fp-green hover:text-fp-green/80 disabled:opacity-40"
                  >✓</button>
                  <button onClick={onClose} className="text-xs text-fp-red hover:text-fp-red/80">✕</button>
                </div>
              ) : (
                <div className="flex items-center gap-1.5 group/name">
                  <span className="text-sm font-medium text-fp-text truncate">
                    {app.display_name || app.app_name}
                  </span>
                  <button
                    onClick={() => onOpenEdit("name")}
                    className="text-fp-muted hover:text-fp-accent opacity-0 group-hover/name:opacity-100 transition-opacity text-xs ml-0.5"
                    title="Rename app"
                  >
                    ✎
                  </button>
                </div>
              )}
              <div
                className="text-xs text-fp-muted truncate max-w-[300px]"
                title={app.executable_path}
              >
                {app.app_name}
              </div>
            </div>
          </div>
        </div>
      </td>

      {/* Today */}
      <td className="px-4 py-3 text-right">
        <span className="text-sm font-medium text-fp-text">
          {formatDuration(app.today_seconds ?? 0)}
        </span>
      </td>

      {/* Total */}
      <td className="px-4 py-3 text-right">
        <span className="text-sm text-fp-muted">
          {formatDuration(app.total_seconds)}
        </span>
      </td>

      {/* Category */}
      <td className="px-4 py-3 text-right">
        {editMode === "category" ? (
          <div className="flex items-center gap-2 justify-end">
            <select
              autoFocus
              defaultValue={app.category}
              onChange={e => onSaveCat(e.target.value as AppCategory)}
              className="bg-fp-card border border-fp-border text-fp-text text-xs rounded-lg px-2 py-1 focus:outline-none focus:border-fp-accent"
            >
              {CATEGORY_LABELS.map(cat => <option key={cat} value={cat}>{cat}</option>)}
            </select>
            <button onClick={onClose} className="text-xs text-fp-red">✕</button>
          </div>
        ) : (
          <button
            onClick={() => !app.is_ignored && onOpenEdit("category")}
            title="Click to change category"
            className={app.is_ignored ? "cursor-default" : "cursor-pointer"}
          >
            <CategoryBadge category={app.category} />
          </button>
        )}
      </td>

      <td className="px-4 py-3 text-right">
        {editingLimit ? (
          <div className="flex items-center justify-end gap-1">
            <input
              autoFocus
              type="number"
              min="1"
              placeholder="mins"
              value={limitInput}
              onChange={(e) => setLimitInput(e.target.value)}
              onKeyDown={async (e) => {
                if (e.key === "Enter") {
                  await useStore.getState().updateAppDailyLimit(
                    app.id,
                    limitInput.trim()
                      ? Number(limitInput)
                      : null
                  );
                  setEditingLimit(false);
                }

                if (e.key === "Escape") {
                  setEditingLimit(false);
                }
              }}
              className="w-16 px-2 py-1 text-xs rounded bg-fp-card border border-fp-border text-fp-text"
            />

            <button
              onClick={async () => {
                await useStore.getState().updateAppDailyLimit(
                  app.id,
                  limitInput.trim()
                    ? Number(limitInput)
                    : null
                );
                setEditingLimit(false);
              }}
              className="text-fp-green text-xs"
            >
              ✓
            </button>
          </div>
        ) : (
          <button
            onClick={() => setEditingLimit(true)}
            className="text-xs text-fp-accent hover:underline  rounded px-2 py-1 bg-fp-accent/0 transition-colors duration-200 hover:bg-fp-accent/10"
          >
            {app.daily_limit_minutes
              ? `${app.daily_limit_minutes}m`
              : "None"}
          </button>
        )}
      </td>

      {/* Reminder */}
      <td className="px-4 py-3 text-right">
        {editingReminder ? (
          <select
            autoFocus
            defaultValue={app.reminder_interval_minutes ?? 0}
            onChange={async (e) => {
              await useStore
                .getState()
                .updateAppReminderInterval(
                  app.id,
                  Number(e.target.value)
                );

              setEditingReminder(false);
            }}
            className="bg-fp-card border border-fp-border text-fp-text text-xs rounded-lg px-2 py-1"
          >
            <option value={0}>Off</option>
            <option value={1}>1m</option>
            <option value={5}>5m</option>
            <option value={10}>10m</option>
            <option value={15}>15m</option>
            <option value={20}>20m</option>
            <option value={25}>25m</option>
            <option value={30}>30m</option>
          </select>
        ) : (
          <button
            onClick={() => setEditingReminder(true)}
            className="text-xs text-fp-accent hover:underline rounded px-2 py-1 hover:bg-fp-accent/10"
          >
            {app.reminder_interval_minutes
              ? `${app.reminder_interval_minutes}m`
              : "Off"}
          </button>
        )}
      </td>

      {/* Soft Lock */}
      <td className="px-4 py-3 text-right">
        {editingSoftLock ? (
          <select
            autoFocus
            defaultValue={app.soft_lock_enabled ? "on" : "off"}
            onChange={async (e) => {
              await useStore
                .getState()
                .updateAppSoftLockEnabled(
                  app.id,
                  e.target.value === "on"
                );

              setEditingSoftLock(false);
            }}
            className="bg-fp-card border border-fp-border text-fp-text text-xs rounded-lg px-2 py-1"
          >
            <option value="off">Off</option>
            <option value="on">On</option>
          </select>
        ) : (
          <button
            onClick={() => setEditingSoftLock(true)}
            className="text-xs text-fp-accent hover:underline rounded px-2 py-1 bg-fp-accent/0 transition-colors duration-200 hover:bg-fp-accent/10"
          >
            {app.soft_lock_enabled ? "On" : "Off"}
          </button>
        )}
      </td>

      <td className="px-4 py-3 text-right">
        {(() => {
          if (app.is_ignored) {
            return (
              <span className="text-xs px-2 py-1 rounded-full bg-fp-border text-fp-muted">
                Ignored
              </span>
            );
          }

          if (!app.daily_limit_minutes || !app.today_seconds) {
            return (
              <span className="text-xs px-2 py-1 rounded-full bg-fp-accent/10 text-fp-accent">
                Normal
              </span>
            );
          }

          const usagePct =
            (app.today_seconds / (app.daily_limit_minutes * 60)) * 100;

          if (usagePct >= 100) {
            return (
              <span className="text-xs px-2 py-1 rounded-full bg-fp-red/10 text-fp-red">
                Over Limit
              </span>
            );
          }

          if (usagePct >= 90) {
            return (
              <span className="text-xs px-2 py-1 rounded-full bg-fp-amber/10 text-fp-amber">
                Near Limit
              </span>
            );
          }

          return (
            <span className="text-xs px-2 py-1 rounded-full bg-fp-green/10 text-fp-green">
              Normal
            </span>
          );
        })()}
      </td>

      {/* Ignore toggle */}
      <td className="px-5 py-3 text-right">
        <button
          onClick={onToggleIgnore}
          className={`w-7 h-7 flex items-center justify-center rounded-lg transition-colors ${app.is_ignored ? "text-fp-amber bg-fp-amber/10 hover:bg-fp-amber/20" : "text-fp-muted hover:text-fp-text hover:bg-fp-border"
            }`}
          title={app.is_ignored ? "Resume tracking" : "Ignore app"}
        >
          {app.is_ignored ? "▷" : "⊘"}
        </button>
      </td>
    </tr>
  );
}
