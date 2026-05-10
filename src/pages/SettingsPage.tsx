import { useState, useEffect } from "react";
import { useStore } from "../store";
import { LoadingSpinner } from "../components/ui";
import { api } from "../utils/api";
import type { Settings } from "../types";

// Apply theme to <html> immediately and persist across reloads
function applyTheme(theme: string) {
  const root = document.documentElement;
  root.classList.remove("dark", "light");
  if (theme === "system") {
    root.classList.add(window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
  } else {
    root.classList.add(theme);
  }
  // Persist so it's applied on next load before React mounts
  localStorage.setItem("fp-theme", theme);
}

export default function SettingsPage() {
  const { settings, fetchSettings, updateSettings } = useStore();
  const [local, setLocal] = useState<Settings | null>(null);
  const [saved, setSaved] = useState(false);
  const [dbStatus, setDbStatus] = useState<{ msg: string; ok: boolean } | null>(null);
  const [dbMoving, setDbMoving] = useState(false);

  useEffect(() => { fetchSettings(); }, []);
  useEffect(() => { if (settings) setLocal({ ...settings }); }, [settings]);

  const set = <K extends keyof Settings>(key: K, val: Settings[K]) => {
    setLocal(prev => {
      if (!prev) return null;
      const next = { ...prev, [key]: val };
      if (key === "theme") applyTheme(val as string);
      return next;
    });
  };

  const save = async () => {
    if (!local) return;
    await updateSettings(local);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleBrowseDb = async () => {
    const folder = await api.pickFolder();
    if (!folder || !local) return;
    const cleanFolder = folder.replace(/\//g, "\\").replace(/\\+$/, "");
    const newPath = `${cleanFolder}\\focuspulse.db`;
    await handleMoveDb(newPath);
  };

  const handleMoveDb = async (newPath: string) => {
    setDbMoving(true);
    setDbStatus(null);
    try {
      const result = await api.moveDatabase(newPath);
      set("database_path", result);
      await fetchSettings();
      setDbStatus({ msg: `✓ Database moved to ${result}`, ok: true });
    } catch (e: any) {
      const msg = String(e?.message ?? e);
      if (msg.includes("same file") || msg.includes("Cannot copy") || msg.includes("already")) {
        setDbStatus({ msg: "Database is already at that location.", ok: true });
      } else {
        setDbStatus({ msg: `Failed: ${msg}`, ok: false });
      }
    } finally {
      setDbMoving(false);
    }
  };

  const handleResetDb = async () => {
    setDbMoving(true);
    setDbStatus(null);
    try {
      const result = await api.resetDatabasePath();
      set("database_path", result);
      await fetchSettings();
      setDbStatus({ msg: `✓ Reset to default: ${result}`, ok: true });
    } catch (e: any) {
      const msg = String(e?.message ?? e);
      if (msg.includes("same file") || msg.includes("Cannot copy") || msg.includes("already")) {
        setDbStatus({ msg: "Database is already at the default location.", ok: true });
      } else {
        setDbStatus({ msg: `Failed: ${msg}`, ok: false });
      }
    } finally {
      setDbMoving(false);
    }
  };

  if (!local) return <LoadingSpinner />;

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 animate-slide-up">

      {/* Tracking */}
      <SettingsSection title="Tracking">
        <SettingsRow label="Polling Interval" description="How often to check the active window">
          <div className="flex items-center gap-2">
            <input type="range" min={250} max={10000} step={250}
              value={local.polling_interval_ms}
              onChange={e => set("polling_interval_ms", Number(e.target.value))}
              className="w-32 accent-fp-accent"
            />
            <span className="text-xs text-fp-text w-16">{local.polling_interval_ms}ms</span>
          </div>
        </SettingsRow>
        <SettingsRow label="Idle Threshold" description="Minutes of inactivity before marking as idle">
          <div className="flex items-center gap-2">
            <input type="number" min={1} max={60}
              value={local.idle_threshold_minutes}
              onChange={e => set("idle_threshold_minutes", Number(e.target.value))}
              className="bg-fp-card border border-fp-border text-fp-text text-sm rounded-lg px-3 py-1.5 w-20 focus:outline-none focus:border-fp-accent"
            />
            <span className="text-xs text-fp-muted">minutes</span>
          </div>
        </SettingsRow>
      </SettingsSection>

      {/* System */}
      <SettingsSection title="System">
        <SettingsRow label="Launch on Startup" description="Start FocusPulse automatically when Windows starts">
          <Toggle
            checked={local.launch_on_startup}
            onChange={async (v) => {
              set("launch_on_startup", v);
              await api.setAutostart(v);
            }}
          />
        </SettingsRow>
        <SettingsRow label="Notifications" description="Show system notifications for daily summaries">
          <Toggle
            checked={local.notification_enabled}
            onChange={async (v) => {
              set("notification_enabled", v);
              // Send a test notification when enabling so user confirms it works
              if (v) await api.sendNotification("FocusPulse", "Notifications enabled ✓");
            }}
          />
        </SettingsRow>
      </SettingsSection>

      {/* Appearance
      <SettingsSection title="Appearance">
        <SettingsRow label="Theme" description="Changes appearance immediately">
          <select
            value={local.theme}
            onChange={e => set("theme", e.target.value as Settings["theme"])}
            className="bg-fp-card border border-fp-border text-fp-text text-sm rounded-lg px-3 py-1.5 focus:outline-none focus:border-fp-accent"
          >
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="system">System Default</option>
          </select>
        </SettingsRow>
      </SettingsSection> */}

      {/* Storage */}
      <SettingsSection title="Storage & Database">
        <div className="px-5 py-4 space-y-3">
          <div>
            <div className="text-sm font-medium text-fp-text mb-0.5">Database Location</div>
            <div className="text-xs text-fp-muted mb-2">
              Your tracking data is stored here. Moving it copies all existing data.
            </div>
            <code className="block bg-fp-bg border border-fp-border rounded-lg p-2 text-xs text-fp-muted break-all font-mono">
              {local.database_path || "Loading..."}
            </code>
          </div>
          <div className="flex gap-2">
            <button
              onClick={handleBrowseDb}
              disabled={dbMoving}
              className="fp-btn-primary text-xs disabled:opacity-50"
            >
              {dbMoving
                ? <span className="flex items-center gap-1.5"><span className="w-3 h-3 border border-white/30 border-t-white rounded-full animate-spin inline-block" />Moving...</span>
                : "🗀 Browse & Move"}
            </button>
            <button onClick={handleResetDb} disabled={dbMoving} className="fp-btn-ghost text-xs disabled:opacity-50">
              ↺ Reset to Default
            </button>
          </div>
          {dbStatus && (
            <div className={`text-xs px-3 py-2 rounded-lg border ${
              dbStatus.ok ? "bg-fp-green/10 text-fp-green border-fp-green/30"
                         : "bg-fp-red/10 text-fp-red border-fp-red/30"
            }`}>
              {dbStatus.msg}
            </div>
          )}
          <p className="text-[11px] text-fp-amber">
            ⚠ After moving the database, restart FocusPulse to ensure tracking uses the new location.
          </p>
        </div>
      </SettingsSection>

      {/* Save */}
      <div className="flex justify-end pt-4 border-t border-fp-border">
        <button onClick={save} className="fp-btn-primary px-8">
          {saved ? "✓ Saved" : "Save Settings"}
        </button>
      </div>
    </div>
  );
}

function SettingsSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h2 className="fp-label mb-3">{title}</h2>
      <div className="fp-card p-0 divide-y divide-fp-border">{children}</div>
    </div>
  );
}

function SettingsRow({ label, description, children }: {
  label: string; description: string; children: React.ReactNode;
}) {
  return (
    <div className="flex items-start justify-between gap-4 px-5 py-4">
      <div className="flex-1">
        <div className="text-sm font-medium text-fp-text">{label}</div>
        <div className="text-xs text-fp-muted mt-0.5">{description}</div>
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`relative w-10 h-5 rounded-full transition-colors duration-200 ${
        checked ? "bg-fp-accent" : "bg-fp-border"
      }`}
    >
      <div className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform duration-200 ${
        checked ? "translate-x-5" : "translate-x-0.5"
      }`} />
    </button>
  );
}
