import { getCurrentWindow } from "@tauri-apps/api/window";

export default function SoftLockPage() {
  return (
    <div className="h-screen flex flex-col items-center justify-center bg-fp-bg text-fp-text">
      <h1 className="text-4xl font-bold mb-4">
        Daily Limit Exceeded
      </h1>

      <p className="text-fp-muted mb-8">
        You have exceeded the allowed usage time.
      </p>

      <button
        onClick={() => getCurrentWindow().close()}
        className="px-5 py-2 rounded-lg bg-fp-accent text-white"
      >
        Continue Anyway
      </button>
    </div>
  );
}