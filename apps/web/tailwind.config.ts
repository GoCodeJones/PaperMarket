import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        // ─── Paleta PaperMarket ───────────────────────────
        bpc: {
          yellow:  "#F5C518",  // amarelo principal
          gold:    "#FFD700",  // amarelo hover
          amber:   "#FFA500",  // destaque laranja
          black:   "#0A0A0A",  // fundo principal
          dark:    "#111111",  // cards
          darker:  "#0D0D0D",  // sidebar
          border:  "#222222",  // bordas padrão
          muted:   "#444444",  // texto secundário
          faint:   "#333333",  // bordas sutis
          // Semânticas
          success: "#00FF88",  // confirmado / liberado
          danger:  "#FF4444",  // erro / rejeitado
          warning: "#FF6600",  // disputa / atenção
          info:    "#4488FF",  // informativo
        },
      },
      fontFamily: {
        mono:    ["'Share Tech Mono'", "'Courier New'", "monospace"],
        display: ["'Bebas Neue'", "sans-serif"],
        body:    ["'Share Tech Mono'", "monospace"],
      },
      fontSize: {
        "2xs": ["0.625rem", { lineHeight: "1rem" }], // 10px
      },
      borderWidth: {
        "3": "3px",
      },
      animation: {
        "mining-shimmer": "shimmer 1.5s infinite linear",
        "blink":          "blink 1s step-end infinite",
        "slide-in":       "slideIn 0.2s ease-out",
        "fade-in":        "fadeIn 0.3s ease-out",
      },
      keyframes: {
        shimmer: {
          "0%":   { backgroundPosition: "200% 0" },
          "100%": { backgroundPosition: "-200% 0" },
        },
        blink: {
          "50%": { opacity: "0" },
        },
        slideIn: {
          "0%":   { transform: "translateY(-8px)", opacity: "0" },
          "100%": { transform: "translateY(0)", opacity: "1" },
        },
        fadeIn: {
          "0%":   { opacity: "0" },
          "100%": { opacity: "1" },
        },
      },
      backgroundImage: {
        "grid-pattern":
          "linear-gradient(#F5C51808 1px, transparent 1px), linear-gradient(90deg, #F5C51808 1px, transparent 1px)",
      },
      backgroundSize: {
        "grid": "32px 32px",
      },
    },
  },
  plugins: [],
};

export default config;