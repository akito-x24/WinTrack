import { useState, useEffect } from "react";
import { useStore } from "../store";
import { CategoryBadge, LoadingSpinner, EmptyState } from "../components/ui";
import { getAppIcon, getAppInitials, formatDuration } from "../utils/helpers";
import { CATEGORY_LABELS, CATEGORY_COLORS } from "../types";
import type { App, AppCategory } from "../types";
import clsx from "clsx";

type EditMode = "category" | "name" | null;

export default function AppBreakdown() {
  const { appList, fetchAppList, updateAppCategory, updateAppDisplayName, setAppIgnored, loading } = useStore();
  const [search, setSearch] = useState("");
  const [catFilter, setCatFilter] = useState<AppCategory | "All">("All");
  const [showIgnored, setShowIgnored] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editMode, setEditMode] = useState<EditMode>(null);

  useEffect(() => { fetchAppList(); }, []);

  const filtered = (appList ?? []).filter(app => {
    if (!showIgnored && app.is_ignored) return false;
    const matchSearch = app.display_name.toLowerCase().includes(search.toLowerCase())
      || app.app_name.toLowerCase().includes(search.toLowerCase());
    const matchCat = catFilter === "All" || app.category === catFilter;
    return matchSearch && matchCat;
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
              className={`text-xs px-3 py-1.5 rounded-full transition-all ${
                catFilter === cat ? "text-white font-medium"
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
            <table className="w-full">
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
  const bgColor = getAppIcon(app.app_name);
  const initials = getAppInitials(app.app_name);
  const [nameInput, setNameInput] = useState(app.display_name);

  useEffect(() => { setNameInput(app.display_name); }, [app.display_name]);

  return (
    <tr className={clsx(
      "group hover:bg-fp-surface/50 transition-colors",
      app.is_ignored && "opacity-50"
    )}>
      <td className="px-5 py-3">
        <div className="flex items-center gap-4">
          {/* Icon */}
          <div
            className="w-9 h-9 rounded-lg flex items-center justify-center text-white text-sm font-medium shrink-0"
            style={{ backgroundColor: bgColor }}
          >
            {initials}
          </div>

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
                  <span className="text-sm font-medium text-fp-text truncate">{app.display_name}</span>
                  {app.display_name !== app.app_name && (
                    <span className="text-xs text-fp-muted truncate">({app.app_name})</span>
                  )}
                  <button
                    onClick={() => onOpenEdit("name")}
                    className="text-fp-muted hover:text-fp-accent opacity-0 group-hover/name:opacity-100 transition-opacity text-xs ml-0.5"
                    title="Rename app"
                  >✎</button>
                </div>
              )}
              <div className="text-xs text-fp-muted truncate max-w-[300px]">{app.executable_path}</div>
            </div>
          </div>
        </div>
      </td>

      {/* Total time */}
      <td className="px-4 py-3 text-right">
        <span className="text-sm font-medium text-fp-text">{formatDuration(app.total_seconds)}</span>
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

      {/* Ignore toggle */}
      <td className="px-5 py-3 text-right">
        <button
          onClick={onToggleIgnore}
          className={`w-7 h-7 flex items-center justify-center rounded-lg transition-colors ${
            app.is_ignored ? "text-fp-amber bg-fp-amber/10 hover:bg-fp-amber/20" : "text-fp-muted hover:text-fp-text hover:bg-fp-border"
          }`}
          title={app.is_ignored ? "Resume tracking" : "Ignore app"}
        >
          {app.is_ignored ? "▶" : "⊘"}
        </button>
      </td>
    </tr>
  );
}