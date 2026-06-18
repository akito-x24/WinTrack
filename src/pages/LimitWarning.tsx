import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../utils/api";

export default function LimitWarning() {
  const params = new URLSearchParams(window.location.search);
  const appName = params.get("app") ?? "Application";
  const processName = params.get("process") ?? "";
  const appId = params.get("appId") ?? "0";
  const currentUsage = parseInt(params.get("currentUsage") ?? "0", 10);
  const dailyLimit = parseInt(params.get("dailyLimit") ?? "0", 10);
  
  const [countdown, setCountdown] = useState(30);
  const [isLocked, setIsLocked] = useState(false);
  const [remainingLockTime, setRemainingLockTime] = useState("");

  const formatDuration = (seconds: number): string => {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    if (h > 0) return `${h}h ${m}m`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
  };

  useEffect(() => {
    // Check if app is currently locked
    const isLockedParam = params.get("isLocked") === "true";
    if (isLockedParam) {
      setIsLocked(true);
      const lockExpiration = params.get("lockExpiration") ?? "";
      if (lockExpiration) {
        const updateLockTime = () => {
          const now = new Date();
          const expiration = new Date(lockExpiration);
          const diffMs = expiration.getTime() - now.getTime();
          if (diffMs > 0) {
            const diffSecs = Math.floor(diffMs / 1000);
            setRemainingLockTime(formatDuration(diffSecs));
          } else {
            setRemainingLockTime("0s");
          }
        };
        updateLockTime();
        const interval = setInterval(updateLockTime, 1000);
        return () => clearInterval(interval);
      }
    }
  }, [params]);

  // Countdown timer
  useEffect(() => {
    if (isLocked) return; // Don't countdown if locked

    if (countdown <= 0) {
      // Auto-close app
      handleCloseApp();
      return;
    }

    const timer = setTimeout(() => {
      setCountdown(countdown - 1);
    }, 1000);

    return () => clearTimeout(timer);
  }, [countdown, isLocked]);

  const handleCloseApp = async () => {
    try {
      if (processName) {
        await api.closeProcess(processName);
      }
      await getCurrentWindow().close();
    } catch (err) {
      console.error("Failed to close app:", err);
    }
  };

  const handleGiveMoreTime = async () => {
    try {
      // Grant 5 more minutes by clearing limit flag
      await api.grantAppMoreTime(parseInt(appId, 10));
      await getCurrentWindow().close();
    } catch (err) {
      console.error("Failed to grant more time:", err);
    }
  };

  const handleWaitForLock = async () => {
    // Just close the warning window, lock remains active
    await getCurrentWindow().close();
  };

  if (isLocked) {
    return (
      <div className="h-screen flex flex-col items-center justify-center bg-gradient-to-br from-fp-bg to-fp-surface text-fp-text px-8">
        <div className="text-6xl mb-4">🔒</div>
        
        <h1 className="text-5xl font-bold mb-2">
          This app is locked
        </h1>

        <p className="text-xl text-fp-muted mb-8 text-center">
          {appName} reached its limit. Try again later.
        </p>

        <div className="bg-fp-card border-2 border-fp-accent rounded-lg p-8 mb-8 text-center">
          <p className="text-fp-muted text-sm mb-2">Lock expires in</p>
          <p className="text-4xl font-bold text-fp-accent">
            {remainingLockTime}
          </p>
        </div>

        <div className="flex gap-4">
          <button
            onClick={handleWaitForLock}
            className="px-6 py-3 rounded-lg bg-fp-card border border-fp-border text-fp-text hover:bg-fp-surface transition"
          >
            [Wait]
          </button>
          <button
            onClick={handleGiveMoreTime}
            className="px-6 py-3 rounded-lg bg-fp-amber text-white font-semibold hover:bg-fp-amber/80 transition"
          >
            [Give Me 5 More Minutes]
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col items-center justify-center bg-gradient-to-br from-fp-bg to-fp-surface text-fp-text px-8">
      {/* Warning icon */}
      <div className="text-7xl mb-6">⚠️</div>

      {/* Title */}
      <h1 className="text-5xl font-bold mb-4 text-center">
        Daily Limit Reached
      </h1>

      {/* App info */}
      <p className="text-2xl text-fp-muted mb-8 text-center">
        You've reached today's limit for <span className="font-semibold text-fp-accent">{appName}</span>
      </p>

      {/* Usage display */}
      <div className="bg-fp-card border-2 border-fp-accent rounded-lg p-8 mb-8 w-full max-w-sm">
        <div className="flex justify-between items-center mb-4">
          <span className="text-fp-muted">Usage</span>
          <span className="text-2xl font-bold">{formatDuration(currentUsage)}</span>
        </div>
        <div className="w-full bg-fp-surface rounded-full h-2">
          <div
            className="bg-fp-accent h-2 rounded-full transition-all"
            style={{
              width: `${Math.min(100, (currentUsage / dailyLimit) * 100)}%`,
            }}
          />
        </div>
        <div className="flex justify-between items-center mt-4 text-sm text-fp-muted">
          <span>Daily Limit</span>
          <span>{formatDuration(dailyLimit)}</span>
        </div>
      </div>

      {/* Countdown */}
      <div className="mb-8 text-center">
        <p className="text-fp-muted text-sm mb-2">App closes in</p>
        <p className="text-6xl font-bold text-fp-accent">
          {countdown}
        </p>
        <p className="text-fp-muted text-sm mt-2">seconds</p>
      </div>

      {/* Buttons */}
      <div className="flex gap-4 w-full max-w-sm">
        <button
          onClick={handleGiveMoreTime}
          className="flex-1 px-6 py-3 rounded-lg bg-fp-amber text-white font-semibold hover:bg-fp-amber/80 transition"
        >
          [Give Me 5 More Minutes]
        </button>
        <button
          onClick={handleCloseApp}
          className="flex-1 px-6 py-3 rounded-lg bg-red-600 text-white font-semibold hover:bg-red-700 transition"
        >
          [Close App]
        </button>
      </div>
    </div>
  );
}
