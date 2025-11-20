/**
 * Master-Class Design Tokens
 * Physics-based, snappy animations with semantic color system
 */

// Physics: Snappy bezier curves for ultra-fast, responsive interactions
export const easings = {
  snappy: [0.18, 0.9, 0.22, 1] as const,
  spring: [0.34, 1.56, 0.64, 1] as const,
  smooth: [0.4, 0, 0.2, 1] as const,
  bounce: [0.68, -0.55, 0.265, 1.55] as const,
} as const;

// Timing: Ultra-fast for instant feedback
export const durations = {
  instant: 100,
  fast: 150,
  normal: 200,
  slow: 300,
  slower: 500,
} as const;

// Semantic Colors: Light & Dark modes
export const colors = {
  light: {
    background: {
      primary: '#ffffff',
      secondary: '#f8f9fa',
      tertiary: '#f1f3f5',
      overlay: 'rgba(0, 0, 0, 0.4)',
    },
    surface: {
      primary: '#ffffff',
      elevated: '#ffffff',
      hover: '#f8f9fa',
      active: '#e9ecef',
    },
    text: {
      primary: '#1a1a1a',
      secondary: '#6b7280',
      tertiary: '#9ca3af',
      inverse: '#ffffff',
    },
    border: {
      default: '#e5e7eb',
      focus: '#3b82f6',
      hover: '#d1d5db',
    },
    accent: {
      primary: '#3b82f6',
      hover: '#2563eb',
      active: '#1d4ed8',
      glow: 'rgba(59, 130, 246, 0.3)',
    },
    highlight: {
      default: '#eff6ff',
      hover: '#dbeafe',
      active: '#bfdbfe',
    },
  },
  dark: {
    background: {
      primary: '#0f172a',
      secondary: '#1e293b',
      tertiary: '#334155',
      overlay: 'rgba(0, 0, 0, 0.6)',
    },
    surface: {
      primary: '#1e293b',
      elevated: '#334155',
      hover: '#334155',
      active: '#475569',
    },
    text: {
      primary: '#f8fafc',
      secondary: '#cbd5e1',
      tertiary: '#94a3b8',
      inverse: '#0f172a',
    },
    border: {
      default: '#334155',
      focus: '#60a5fa',
      hover: '#475569',
    },
    accent: {
      primary: '#60a5fa',
      hover: '#3b82f6',
      active: '#2563eb',
      glow: 'rgba(96, 165, 250, 0.3)',
    },
    highlight: {
      default: '#1e3a5f',
      hover: '#1e40af',
      active: '#1e3a8a',
    },
  },
} as const;

// Typography
export const typography = {
  fontFamily: {
    sans: ['Inter', 'SF Pro Display', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
    mono: ['SF Mono', 'Monaco', 'Cascadia Code', 'monospace'],
  },
  fontSize: {
    xs: '0.75rem',
    sm: '0.875rem',
    base: '1rem',
    lg: '1.125rem',
    xl: '1.25rem',
    '2xl': '1.5rem',
    '3xl': '1.875rem',
  },
  fontWeight: {
    normal: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },
  lineHeight: {
    tight: 1.2,
    normal: 1.5,
    relaxed: 1.75,
  },
} as const;

// Spacing
export const spacing = {
  xs: '0.25rem',
  sm: '0.5rem',
  md: '1rem',
  lg: '1.5rem',
  xl: '2rem',
  '2xl': '3rem',
} as const;

// Shadows
export const shadows = {
  sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
  md: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
  lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
  xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
  glow: '0 0 20px rgba(59, 130, 246, 0.3)',
} as const;

// Z-index layers
export const zIndex = {
  base: 0,
  dropdown: 100,
  overlay: 200,
  modal: 300,
  tooltip: 400,
} as const;

// Command Palette specific
export const commandPalette = {
  width: '640px',
  maxHeight: '600px',
  borderRadius: '12px',
  itemHeight: '48px',
  itemPadding: '12px 16px',
  inputHeight: '56px',
  previewWidth: '400px',
} as const;
