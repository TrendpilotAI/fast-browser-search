/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        cmd: {
          bg: '#0B0E11',
          surface: '#12161C',
          hover: '#1A1F27',
          active: '#1F2530',
          border: 'rgba(255,255,255,0.06)',
          accent: '#4C8EFF',
          'accent-light': '#A7C2FF',
          text: '#F6F7FB',
          'text-dim': 'rgba(246,247,251,0.70)',
          'text-muted': 'rgba(246,247,251,0.45)',
        }
      },
      animation: {
        'in': 'fadeIn 80ms cubic-bezier(0.18, 0.9, 0.22, 1)',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0', transform: 'translateY(-10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        }
      }
    },
  },
  plugins: [],
}