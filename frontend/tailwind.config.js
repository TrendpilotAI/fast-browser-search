import { tokens } from './src/ui/tokens';

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        bg: {
          primary: tokens.colors.backgroundPrimary,
          elevated: tokens.colors.backgroundElevated,
        },
        surface: {
          DEFAULT: tokens.colors.surface,
          hover: tokens.colors.surfaceHover,
          active: tokens.colors.surfaceActive,
        },
        border: {
          subtle: tokens.colors.borderSubtle,
          focus: tokens.colors.borderFocus,
        },
        text: {
          primary: tokens.colors.textPrimary,
          secondary: tokens.colors.textSecondary,
          muted: tokens.colors.textMuted,
        },
        accent: {
          primary: tokens.colors.accentPrimary,
          tint: tokens.colors.accentTint,
        }
      },
      borderRadius: {
        md: tokens.radius.md,
        lg: tokens.radius.lg,
        pill: tokens.radius.pill,
      },
      boxShadow: {
        card: tokens.shadow.card,
        focus: tokens.shadow.focus,
      },
      transitionTimingFunction: {
        snappy: tokens.motion.snappy,
      }
    },
  },
  plugins: [],
}
