/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Light mode colors
        'bg-primary': '#ffffff',
        'bg-secondary': '#f8f9fa',
        'bg-tertiary': '#f1f3f5',
        'surface-primary': '#ffffff',
        'surface-elevated': '#ffffff',
        'surface-hover': '#f8f9fa',
        'surface-active': '#e9ecef',
        'text-primary': '#1a1a1a',
        'text-secondary': '#6b7280',
        'text-tertiary': '#9ca3af',
        'border-default': '#e5e7eb',
        'border-focus': '#3b82f6',
        'border-hover': '#d1d5db',
        'accent-primary': '#3b82f6',
        'accent-hover': '#2563eb',
        'accent-active': '#1d4ed8',
        'highlight-default': '#eff6ff',
        'highlight-hover': '#dbeafe',
        'highlight-active': '#bfdbfe',
      },
      fontFamily: {
        sans: ['Inter', 'SF Pro Display', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['SF Mono', 'Monaco', 'Cascadia Code', 'monospace'],
      },
      transitionTimingFunction: {
        'snappy': 'cubic-bezier(0.18, 0.9, 0.22, 1)',
        'spring': 'cubic-bezier(0.34, 1.56, 0.64, 1)',
        'bounce': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
      },
      transitionDuration: {
        'instant': '100ms',
        'fast': '150ms',
        'normal': '200ms',
        'slow': '300ms',
        'slower': '500ms',
      },
      boxShadow: {
        'glow': '0 0 20px rgba(59, 130, 246, 0.3)',
      },
    },
  },
  plugins: [],
}