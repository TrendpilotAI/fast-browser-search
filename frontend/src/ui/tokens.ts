export const tokens = {
  colors: {
    backgroundPrimary: "#0B0E11",
    backgroundElevated: "#12161C",
    surface: "#12161C",
    surfaceHover: "#1A1F27",
    surfaceActive: "#1F2530",
    borderSubtle: "rgba(255,255,255,0.06)",
    borderFocus: "#4C8EFF",
    textPrimary: "#F6F7FB",
    textSecondary: "rgba(246,247,251,0.70)",
    textMuted: "rgba(246,247,251,0.45)",
    accentPrimary: "#4C8EFF",
    accentTint: "#A7C2FF"
  },
  radius: { md: "12px", lg: "16px", pill: "9999px" },
  shadow: {
    card: "0px 6px 20px rgba(0,0,0,0.35)",
    focus: "0 0 0 2px rgba(76,142,255,0.75)"
  },
  motion: {
    instant: "80ms",
    // The "Snappy" bezier from the plan
    snappy: "cubic-bezier(0.18, 0.9, 0.22, 1)",
    spring: {
        type: "spring",
        stiffness: 300,
        damping: 30
    }
  },
  spacing: { xs: "8px", sm: "12px", md: "16px", lg: "24px" }
};

