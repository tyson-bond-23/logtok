module.exports = {
  darkMode: 'class',
  content: ['./templates/ui/**/*.html', './src/ui/**/*.rs'],
  theme: {
    extend: {
      colors: {
        brand: { 50: '#eef2ff', 100: '#e0e7ff', 400: '#818cf8', 500: '#6366f1', 600: '#4f46e5', 700: '#4338ca', 900: '#312e81' },
        surface: { 50: '#fafaf5', 100: '#f4f4f5', 200: '#e4e4e7', 700: '#18181b', 800: '#111113', 900: '#0f0f10', 950: '#050506' },
      },
      fontFamily: {
        sans: ['system-ui', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', '"Helvetica Neue"', 'sans-serif'],
        mono: ['ui-monospace', '"Cascadia Code"', '"Fira Code"', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
      },
    }
  },
  plugins: [],
}
