import { useEffect } from "react";
import { useStore } from "./store";
import Sidebar from "./components/layout/Sidebar";
import Header from "./components/layout/Header";
import Dashboard from "./pages/Dashboard";
import DailyAnalytics from "./pages/DailyAnalytics";
import WeeklyAnalytics from "./pages/WeeklyAnalytics";
import MonthlyAnalytics from "./pages/MonthlyAnalytics";
import AppBreakdown from "./pages/AppBreakdown";
import TimelineView from "./pages/TimelineView";
import SettingsPage from "./pages/SettingsPage";
import ExportCenter from "./pages/ExportCenter";

// 🔍 Component to handle Theme changes
function ThemeHandler() {
  const theme = useStore(s => s.settings?.theme);

  useEffect(() => {
    const root = document.documentElement;
    
    // Clean slate
    root.classList.remove("dark");

    if (theme === "dark") {
      root.classList.add("dark");
    } else if (theme === "system") {
      // Check system preference
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      if (mediaQuery.matches) root.classList.add("dark");
      
      // Listen for changes
      const handler = (e: MediaQueryListEvent) => {
        if (e.matches) root.classList.add("dark");
        else root.classList.remove("dark");
      };
      mediaQuery.addEventListener("change", handler);
      return () => mediaQuery.removeEventListener("change", handler);
    }
    // If "light", do nothing (keep dark class off)
  }, [theme]);

  return null;
}

export default function App() {
  const { view, refreshAll, fetchSettings, fetchAppList } = useStore();

  useEffect(() => {
    refreshAll();
    fetchSettings();
    fetchAppList();
  }, []);

  useEffect(() => {
    const interval = setInterval(() => refreshAll(), 30_000);
    return () => clearInterval(interval);
  }, []);

  
  const renderPage = () => {
    switch (view) {
      case "dashboard": return <Dashboard />;
      case "daily":     return <DailyAnalytics />;
      case "weekly":    return <WeeklyAnalytics />;
      case "monthly":   return <MonthlyAnalytics />;
      case "apps":      return <AppBreakdown />;
      case "timeline":  return <TimelineView />;
      case "settings":  return <SettingsPage />;
      case "export":    return <ExportCenter />;
      default:          return <Dashboard />;
    }
  };

  return (
    <div className="flex h-screen bg-fp-bg text-fp-text font-sans">
      <ThemeHandler /> {/* 👈 Injected here */}
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header />
        <main className="flex-1 overflow-y-auto p-6">
          {renderPage()}
        </main>
      </div>
    </div>
  );
}
