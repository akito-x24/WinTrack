import { getCurrentWindow } from "@tauri-apps/api/window";

export default function SoftLockPage() {
  const params = new URLSearchParams(window.location.search);

  const appName =
    params.get("app") ?? "This application";

  const processName =
    params.get("process") ?? "";

  return (
    <div className="h-screen flex flex-col items-center justify-center bg-fp-bg text-fp-text">
      <h1 className="text-4xl font-bold mb-4">
        Daily Limit Exceeded
      </h1>

      <p className="text-fp-muted mb-8">
        {appName} has exceeded its daily limit.
      </p>

      <button
        onClick={() => {
          console.log("CONTINUE CLICKED");
          getCurrentWindow().close();
        }}
        className="px-5 py-2 rounded-lg bg-fp-accent text-white"
      >
        Continue Anyway
      </button>
      <button
        onClick={async () => {
          const { api } = await import("../utils/api");

          await api.closeProcess(processName);

          await getCurrentWindow().close();
        }}
        className="px-5 py-2 rounded-lg bg-red-600 text-white ml-3"
      >
        Close App
      </button>
    </div>
  );
}