import { useState, useEffect } from "react";
import { useStore } from "../store";
import { LoadingSpinner } from "../components/ui";
import { api } from "../utils/api";
import type { Settings } from "../types";

export default function SettingsPage() {
  const {
    settings,
    fetchSettings, 
    updateSettings,
    refreshAll,
    fetchAppList,
  } = useStore();
  const [local, setLocal] = useState<Settings | null>(null);
  const [saved, setSaved] = useState(false);
  const [pendingReset, setPendingReset] = useState<
    "none" | "reset" | "factory"
  >("none");

  useEffect(() => { fetchSettings(); }, []);
  useEffect(() => { if (settings) setLocal({ ...settings }); }, [settings]);

  const set = <K extends keyof Settings>(key: K, val: Settings[K]) => {
    setLocal(prev => {
      if (!prev) return null;
      const next = { ...prev, [key]: val };
      return next;
    });
  };

  const save = async () => {
    if (!local) return;

    await api.setAutostart(local.launch_on_startup);

    await updateSettings(local);

    if (pendingReset === "reset") {
      await api.resetTrackingData();

      await Promise.all([
        refreshAll(),
        fetchAppList(),
        fetchSettings(),
      ]);

      setPendingReset("none");
    }

    if (pendingReset === "factory") {
      await api.factoryReset();

      await Promise.all([
        refreshAll(),
        fetchAppList(),
        fetchSettings(),
      ]);

      setPendingReset("none");

      window.location.reload();
    }

    if (local.notification_enabled) {
      await api.sendNotification(
        "WinTrack",
        "Settings saved successfully"
      );
    }
  };

  const resetTrackingData = () => {
    setPendingReset(prev =>
      prev === "reset" ? "none" : "reset"
    );
  };

  const factoryReset = () => {
    setPendingReset(prev =>
      prev === "factory" ? "none" : "factory"
    );
  };

  if (!local) return <LoadingSpinner />;

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">

      {/* Tracking */}
      <SettingsSection title="Tracking">
        <SettingsRow label="Polling Interval" description="How often to check the active window">
          <div className="flex items-center gap-2">
            <select
              value={local.polling_interval_ms}
              onChange={(e) =>
                set("polling_interval_ms", Number(e.target.value))
              }
              className="bg-wt-card border border-wt-border text-wt-text text-sm rounded-lg px-3 py-1.5"
            >
              <option value={1000}>1 second</option>
              <option value={2000}>2 seconds</option>
              <option value={3000}>3 seconds</option>
              <option value={5000}>5 seconds</option>
              <option value={10000}>10 seconds</option>
            </select>
          </div>
        </SettingsRow>
        <SettingsRow label="Idle Threshold" description="Minutes of inactivity before marking as idle">
          <div className="flex items-center gap-2">
            <select
              value={local.idle_threshold_minutes}
              onChange={(e) =>
                set(
                  "idle_threshold_minutes",
                  Number(e.target.value)
                )
              }
              className="bg-wt-card border border-wt-border text-wt-text text-sm rounded-lg px-3 py-1.5"
            >
              <option value={0}>Never</option>
              <option value={5}>5 minutes</option>
              <option value={10}>10 minutes</option>
              <option value={15}>15 minutes</option>
              <option value={20}>20 minutes</option>
              <option value={25}>25 minutes</option>
              <option value={30}>30 minutes</option>
            </select>
          </div>
        </SettingsRow>
      </SettingsSection>

      {/* System */}
      <SettingsSection title="System">
        <SettingsRow label="Launch on Startup" description="Start WinTrack automatically when Windows starts">
          <Toggle
            checked={local.launch_on_startup}
            onChange={(v) => set("launch_on_startup", v)}
          />
        </SettingsRow>
        <SettingsRow
          label="Start Minimized"
          description="Launch directly to the system tray"
        >
          <Toggle
            checked={local.start_minimized}
            onChange={(v) => set("start_minimized", v)}
          />
        </SettingsRow>
        <SettingsRow label="Notifications" description="Show system notifications for daily summaries">
          <Toggle
            checked={local.notification_enabled}
            onChange={(v) => set("notification_enabled", v)}
          />
        </SettingsRow>
      </SettingsSection>

      <SettingsSection title="Danger Zone">
        <SettingsRow
          label="Reset All Data"
          description="Delete all tracking history while keeping settings"
        >
          <button
            onClick={resetTrackingData}
            className={`px-3 py-1.5 rounded-lg border text-sm transition-colors ${pendingReset === "reset"
                ? "border-yellow-500 text-yellow-400 bg-yellow-500/10"
                : "border-yellow-500 text-yellow-400 hover:bg-yellow-500/10"
              }`}
          >
            {pendingReset === "reset"
              ? "✓ Reset Queued"
              : "Reset Data"}
          </button>
        </SettingsRow>

        <SettingsRow
          label="Factory Reset"
          description="Delete all tracking history and settings"
        >
          <button
            onClick={factoryReset}
            className={`px-3 py-1.5 rounded-lg border text-sm transition-colors ${pendingReset === "factory"
                ? "border-red-500 text-red-400 bg-red-500/10"
                : "border-red-500 text-red-400 hover:bg-red-500/10"
              }`}
          >
            {pendingReset === "factory"
              ? "✓ Factory Reset Queued"
              : "Factory Reset"}
          </button>
        </SettingsRow>
      </SettingsSection>

            {pendingReset !== "none" && (
        <div className="wt-card border border-red-500/30 bg-red-500/10 p-4">
          <div className="text-red-400 font-medium">
            ⚠ Pending Action
          </div>

          <div className="text-sm text-wt-muted mt-1">
            {pendingReset === "reset"
              ? "All tracking history will be deleted when you click Save Settings."
              : "All tracking history and settings will be deleted when you click Save Settings."}
          </div>
        </div>
      )}

      {/* Save */}
      <div className="flex justify-end pt-4 border-t border-wt-border">
        <button onClick={save} className="wt-btn-primary px-8">
          {saved ? "✓ Saved" : "Save Settings"}
        </button>
      </div>
    </div>
  );
}

function SettingsSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h2 className="wt-label mb-3">{title}</h2>
      <div className="wt-card p-0 divide-y divide-wt-border">{children}</div>
    </div>
  );
}

function SettingsRow({ label, description, children }: {
  label: string; description: string; children: React.ReactNode;
}) {
  return (
    <div className="flex items-start justify-between gap-4 px-5 py-4">
      <div className="flex-1">
        <div className="text-sm font-medium text-wt-text">{label}</div>
        <div className="text-xs text-wt-muted mt-0.5">{description}</div>
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`relative w-10 h-5 rounded-full transition-colors duration-200 ${checked ? "bg-wt-accent" : "bg-wt-border"
        }`}
    >
      <div className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform duration-200 ${checked ? "translate-x-5" : "translate-x-0.5"
        }`} />
    </button>
  );
}
