import { useEffect, useState, Suspense, lazy } from "react";
import { useStore } from "./store";
import Sidebar from "./components/layout/Sidebar";
import Header from "./components/layout/Header";
import Dashboard from "./pages/Dashboard";
import DailyAnalytics from "./pages/DailyAnalytics";
import WeeklyAnalytics from "./pages/WeeklyAnalytics";
import MonthlyAnalytics from "./pages/MonthlyAnalytics";
import AppBreakdown from "./pages/AppBreakdown";
import TimelineView from "./pages/TimelineView";
import { getCurrentWindow } from "@tauri-apps/api/window";
import SoftLockPage from "./pages/SoftLockPage";
import { LoadingSpinner } from "./components/ui";

// Not needed on first paint and don't share Recharts with the analytics
// pages, so they're split into their own chunks rather than bundled with
// the default Dashboard route.
const SettingsPage = lazy(() => import("./pages/SettingsPage"));
const ExportCenter = lazy(() => import("./pages/ExportCenter"));

export default function App() {
  const { view, refreshAll, fetchSettings, fetchAppList } = useStore();
  const [isSoftLockWindow, setIsSoftLockWindow] = useState(false);

  useEffect(() => {
    const disableContextMenu = (e: MouseEvent) => {
      e.preventDefault();
    };

    document.addEventListener("contextmenu", disableContextMenu);

    return () => {
      document.removeEventListener("contextmenu", disableContextMenu);
    };
  }, []);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) {
      return;
    }

    const label = getCurrentWindow().label;

    if (label.startsWith("soft-lock")) {
      setIsSoftLockWindow(true);
    }
  }, []);

  useEffect(() => {
    if (isSoftLockWindow) return;

    refreshAll();
    fetchSettings();
    fetchAppList();
  }, [isSoftLockWindow]);

  useEffect(() => {
    if (isSoftLockWindow) return;

    const interval = setInterval(() => refreshAll(), 30_000);
    return () => clearInterval(interval);
  }, [isSoftLockWindow]);


  const renderPage = () => {
    switch (view) {
      case "dashboard": return <Dashboard />;
      case "daily": return <DailyAnalytics />;
      case "weekly": return <WeeklyAnalytics />;
      case "monthly": return <MonthlyAnalytics />;
      case "apps": return <AppBreakdown />;
      case "timeline": return <TimelineView />;
      case "settings": return <SettingsPage />;
      case "export": return <ExportCenter />;
      default: return <Dashboard />;
    }
  };

  if (isSoftLockWindow) {
    return <SoftLockPage />;
  }

  return (
    <div className="flex h-screen bg-fp-bg text-fp-text font-sans">
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header />
        <main className="flex-1 overflow-y-auto p-6">
          <Suspense fallback={<LoadingSpinner />}>
            {renderPage()}
          </Suspense>
        </main>
      </div>
    </div>
  );
}