import { useState } from "react";
import { api } from "../utils/api";
import { todayString, subtractDays } from "../utils/helpers";

export default function ExportCenter() {
  const [format, setFormat] = useState<"csv" | "json">("csv");
  const [startDate, setStartDate] = useState(subtractDays(todayString(), 7));
  const [endDate, setEndDate] = useState(todayString());
  const [outputPath, setOutputPath] = useState("");
  const [status, setStatus] = useState<"idle" | "picking" | "exporting" | "done" | "error">("idle");
  const [message, setMessage] = useState("");

  const handleBrowseOutput = async () => {
    setStatus("picking");
    const path = await api.pickSavePath(format);
    if (path) {
      // Strip extension - we append it ourselves so format switch stays consistent
      setOutputPath(path.replace(/\.(csv|json)$/i, ""));
    }
    setStatus("idle");
  };

  const doExport = async () => {
    // If no path chosen yet, open the picker first
    let finalPath = outputPath;
    if (!finalPath.trim()) {
      setStatus("picking");
      const picked = await api.pickSavePath(format);
      if (!picked) { setStatus("idle"); return; }
      finalPath = picked.replace(/\.(csv|json)$/i, "");
      setOutputPath(finalPath);
      setStatus("idle");
    }

    setStatus("exporting");
    setMessage("");
    try {
      const fullPath = `${finalPath}.${format}`;
      const result = await api.exportData(format, startDate, endDate, fullPath);
      setStatus("done");
      setMessage(`✓ Exported to: ${result}`);
    } catch (e: any) {
      setStatus("error");
      setMessage(`Export failed: ${e?.message ?? String(e)}`);
    }
  };

  const isBusy = status === "exporting" || status === "picking";

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">
      <div className="fp-card space-y-5">
        <h2 className="text-sm font-semibold text-fp-text">Export Usage Data</h2>

        {/* Format */}
        <div>
          <label className="fp-label block mb-2">Format</label>
          <div className="flex gap-2">
            {(["csv", "json"] as const).map(f => (
              <button
                key={f}
                onClick={() => setFormat(f)}
                className={`px-4 py-2 rounded-lg text-sm font-medium border transition-all ${
                  format === f
                    ? "bg-fp-accent text-white border-fp-accent"
                    : "bg-fp-card text-fp-muted border-fp-border hover:text-fp-text"
                }`}
              >
                {f.toUpperCase()}
              </button>
            ))}
          </div>
        </div>

        {/* Date range */}
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="fp-label block mb-2">Start Date</label>
            <input
              type="date" value={startDate} max={endDate}
              onChange={e => setStartDate(e.target.value)}
              className="w-full bg-fp-card border border-fp-border text-fp-text text-sm rounded-lg px-3 py-2 focus:outline-none focus:border-fp-accent"
            />
          </div>
          <div>
            <label className="fp-label block mb-2">End Date</label>
            <input
              type="date" value={endDate} min={startDate} max={todayString()}
              onChange={e => setEndDate(e.target.value)}
              className="w-full bg-fp-card border border-fp-border text-fp-text text-sm rounded-lg px-3 py-2 focus:outline-none focus:border-fp-accent"
            />
          </div>
        </div>

        {/* Output path */}
        <div>
          <label className="fp-label block mb-2">Output File</label>
          <div className="flex gap-2">
            <input
              type="text"
              value={outputPath ? `${outputPath}.${format}` : ""}
              readOnly
              placeholder="Click Browse or Export Data to choose location..."
              className="flex-1 bg-fp-bg border border-fp-border text-fp-muted text-xs rounded-lg px-3 py-2 font-mono focus:outline-none cursor-default"
            />
            <button
              onClick={handleBrowseOutput}
              disabled={isBusy}
              className="fp-btn-ghost text-xs shrink-0 disabled:opacity-50"
            >
              🗀 Browse
            </button>
          </div>
        </div>

        {/* Quick presets */}
        <div>
          <label className="fp-label block mb-2">Quick Presets</label>
          <div className="flex gap-2 flex-wrap">
            {[{ label: "Last 7 days", days: 7 }, { label: "Last 30 days", days: 30 }, { label: "Last 90 days", days: 90 }].map(p => (
              <button
                key={p.label}
                onClick={() => { setStartDate(subtractDays(todayString(), p.days)); setEndDate(todayString()); }}
                className="fp-btn-ghost text-xs"
              >
                {p.label}
              </button>
            ))}
          </div>
        </div>

        {/* Export button */}
        <button
          onClick={doExport}
          disabled={isBusy}
          className="fp-btn-primary w-full flex items-center justify-center gap-2 disabled:opacity-60"
        >
          {status === "picking" ? (
            <><div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />Choosing location...</>
          ) : status === "exporting" ? (
            <><div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />Exporting...</>
          ) : "Export Data"}
        </button>

        {message && (
          <div className={`text-xs px-3 py-2 rounded-lg ${
            status === "done" ? "bg-fp-green/15 text-fp-green" : "bg-fp-red/15 text-fp-red"
          }`}>
            {message}
          </div>
        )}
      </div>

      <div className="fp-card bg-fp-accent/5 border-fp-accent/20">
        <h3 className="text-xs font-semibold text-fp-accent mb-2">Export Info</h3>
        <ul className="text-xs text-fp-muted space-y-1">
          <li>• Click <strong className="text-fp-text">Export Data</strong> - a save dialog will open if no location is set</li>
          <li>• CSV exports are compatible with Excel, Google Sheets, etc.</li>
          <li>• JSON exports include all session metadata</li>
          <li>• All data stays local - nothing is uploaded anywhere</li>
        </ul>
      </div>
    </div>
  );
}
