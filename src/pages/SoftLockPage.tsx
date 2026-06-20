import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppIcon } from "../components/ui";
import { api } from "../utils/api";
import { formatDuration } from "../utils/helpers";

const COUNTDOWN_SECONDS = 30;
 
// Mirrors the PWA_SCHEME constant in src-tauri/src/browser_pwa.rs. Browser
// PWAs are stable-identified this way; everything else is a plain exe path.
const PWA_SCHEME = "wintrack-pwa://";

export default function SoftLockPage() {
  const params = new URLSearchParams(window.location.search);
  const appId = Number.parseInt(params.get("appId") ?? "0", 10);
  const identifier = params.get("identifier") ?? "";
  const processName = params.get("process") ?? "";
  const initialAppName = params.get("app") ?? "This application";
  const currentUsage = Number.parseInt(params.get("currentUsage") ?? "0", 10);
  const dailyLimit = Number.parseInt(params.get("dailyLimit") ?? "0", 10);
  const isPwa = identifier.startsWith(PWA_SCHEME);

  const [appName, setAppName] = useState(initialAppName);
  const [iconData, setIconData] = useState<string | null>(null);
  const [countdown, setCountdown] = useState(COUNTDOWN_SECONDS);
  const [closeError, setCloseError] = useState<string | null>(null);
  const actionStarted = useRef(false);

  useEffect(() => {
    if (!appId) return;

    api.getSoftLockAppDetails(appId)
      .then((details) => {
        setAppName(details.display_name || initialAppName);
        setIconData(details.icon_data ?? null);
      })
      .catch((err) => {
        console.error("Failed to load soft lock app details:", err);
      });
  }, [appId, initialAppName]);

  const closeWarningWindow = async () => {
    if (appId) {
      await api.finishSoftLockWarning(appId).catch((err) => {
        console.error("Failed to finish soft lock warning:", err);
      });
    }
    await getCurrentWindow().close();
  };

  // Manual "Close App" click. This is allowed to attempt closing the app
  // for any app type, including PWAs - the backend only ever closes the
  // specific PWA window (never the whole browser), or reports failure.
  const closeTargetApp = async () => {
    if (actionStarted.current) return;
    actionStarted.current = true;

    try {
      if (processName) {
        await api.closeProcess(identifier, processName);
      }
      await closeWarningWindow();
    } catch (err) {
      console.error("Failed to close app:", err);
      // Could not safely close (most likely a PWA whose window couldn't be
      // located). Never fall back to a destructive action - leave the
      // warning up so the user can retry, wait it out, or take +5 minutes.
      actionStarted.current = false;
      setCloseError(
        isPwa
          ? "Couldn't close this app's window automatically. You can close it yourself, or take 5 more minutes."
          : "Couldn't close this app automatically. You can close it yourself, or take 5 more minutes."
      );
    }
  };

  const giveMoreTime = async () => {
    if (actionStarted.current) return;
    actionStarted.current = true;

    try {
      if (appId) {
        await api.grantAppMoreTime(appId);
      }
    } catch (err) {
      console.error("Failed to grant more time:", err);
    } finally {
      await getCurrentWindow().close();
    }
  };

  useEffect(() => {
    const timer = window.setInterval(() => {
      setCountdown((value) => Math.max(0, value - 1));
    }, 1000);

    return () => window.clearInterval(timer);
  }, []);

  // On countdown expiry: auto-close is only safe for apps we can target by
  // process (Win32/UWP). For browser PWAs, closing "the process" means the
  // entire browser - every window, tab, and profile - so we never do that
  // automatically. The warning simply stays up; the user must choose
  // "Close App" (targeted close) or "+5 minutes" themselves.
  useEffect(() => {
    if (countdown !== 0 || actionStarted.current) return;
    if (isPwa) return;
    void closeTargetApp();
  }, [countdown]);

  return (
    <div className="h-screen w-screen overflow-hidden bg-wt-bg text-wt-text">
      <div className="flex min-h-screen flex-col items-center justify-center px-8 py-10">
        <div className="mb-8 flex flex-col items-center gap-5 text-center">
          <div className="rounded-lg border border-wt-border bg-wt-card p-5 shadow-2xl shadow-black/30">
            <AppIcon
              name={appName}
              iconData={iconData}
              className="h-24 w-24 rounded-lg"
            />
          </div>

          <div>
            <p className="mb-3 text-sm font-medium uppercase tracking-wider text-wt-muted">
              Daily Limit Reached
            </p>
            <h1 className="max-w-4xl text-balance text-4xl font-semibold leading-tight md:text-6xl">
              You have reached today's limit for {appName}
            </h1>
          </div>
        </div>

        <div className="mb-8 grid w-full max-w-3xl grid-cols-1 gap-4 md:grid-cols-3">
          <div className="rounded-lg border border-wt-border bg-wt-card px-6 py-5 text-center">
            <p className="mb-2 text-xs font-medium uppercase tracking-wider text-wt-muted">
              Today's Usage
            </p>
            <p className="text-3xl font-semibold">
              {formatDuration(currentUsage)}
            </p>
          </div>

          <div className="rounded-lg border border-wt-border bg-wt-card px-6 py-5 text-center">
            <p className="mb-2 text-xs font-medium uppercase tracking-wider text-wt-muted">
              Daily Limit
            </p>
            <p className="text-3xl font-semibold">
              {formatDuration(dailyLimit)}
            </p>
          </div>

          <div className="rounded-lg border border-wt-accent/60 bg-wt-accent/10 px-6 py-5 text-center">
            <p className="mb-2 text-xs font-medium uppercase tracking-wider text-wt-muted">
              {isPwa ? "Please close manually" : "Closing in"}
            </p>
            <p className="text-5xl font-bold text-wt-accent">
              {isPwa ? "—" : countdown}
            </p>
          </div>
        </div>

        {closeError && (
          <div className="mb-4 w-full max-w-xl rounded-lg border border-wt-accent/40 bg-wt-accent/10 px-4 py-3 text-center text-sm text-wt-text">
            {closeError}
          </div>
        )}

        <div className="flex w-full max-w-xl flex-col gap-3 sm:flex-row">
          <button
            onClick={closeTargetApp}
            className="flex-1 rounded-lg bg-red-600 px-6 py-4 text-sm font-semibold text-white transition hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-300"
          >
            Close App
          </button>

          <button
            onClick={giveMoreTime}
            className="flex-1 rounded-lg bg-wt-accent px-6 py-4 text-sm font-semibold text-white transition hover:opacity-90 focus:outline-none focus:ring-2 focus:ring-blue-300"
          >
            Give Me 5 More Minutes
          </button>
        </div>
      </div>
    </div>
  );
}