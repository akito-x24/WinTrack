/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // WinTrack design tokens
        fp: {
          bg: "#0d0f14",
          surface: "#13161e",
          card: "#1a1e28",
          border: "#252a36",
          text: "#e2e8f0",
          muted: "#64748b",
          accent: "#3b82f6",
          "accent-glow": "#1d4ed8",
          green: "#22c55e",
          amber: "#f59e0b",
          red: "#ef4444",
          purple: "#8b5cf6",
          cyan: "#06b6d4",
          // Category colors
          productive: "#22c55e",
          entertainment: "#f59e0b",
          social: "#8b5cf6",
          gaming: "#ef4444",
          development: "#3b82f6",
          study: "#06b6d4",
          other: "#64748b",
        },
      },
      fontFamily: {
        sans: ["'DM Sans'", "system-ui", "sans-serif"],
        mono: ["'JetBrains Mono'", "monospace"],
        display: ["'DM Sans'", "sans-serif"],
      },
      backgroundImage: {
        "grid-pattern":
          "radial-gradient(circle, #252a36 1px, transparent 1px)",
      },
      animation: {
        "fade-in": "fadeIn 0.2s ease-out",
        "slide-up": "slideUp 0.3s ease-out",
        "pulse-slow": "pulse 3s ease-in-out infinite",
        "bar-fill": "barFill 0.6s ease-out forwards",
      },
      keyframes: {
        fadeIn: { from: { opacity: 0 }, to: { opacity: 1 } },
        slideUp: {
          from: { opacity: 0, transform: "translateY(8px)" },
          to: { opacity: 1, transform: "translateY(0)" },
        },
        barFill: {
          from: { width: "0%" },
          to: { width: "var(--bar-width)" },
        },
      },
    },
  },
  plugins: [],
};
