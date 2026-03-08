/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        cyber: {
          bg: '#12101a',
          panel: '#191b23',
          card: '#1c1e26',
          'card-hover': '#262a37',
          cyan: '#00ffff',
          magenta: '#ff00ff',
          'neon-green': '#00ff88',
          'neon-red': '#ff3366',
          'neon-orange': '#ff8800',
          purple: '#5000c8',
          'dim-purple': '#786496',
          'light-purple': '#c8b4ff',
          'electric-blue': '#00d4ff',
        }
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      }
    },
  },
  plugins: [],
}
