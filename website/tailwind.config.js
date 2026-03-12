/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: '#030304',
        surface: '#0F1115',
        foreground: '#FFFFFF',
        muted: '#94A3B8',
        border: '#1E293B',
        primary: '#00D2FF',    // Bright Cyan/Teal
        secondary: '#805AD7',  // Purple
        tertiary: '#FF42A1',   // Pink/Magenta
        accent: '#3A7BD5',     // Soft Blue
      },
      fontFamily: {
        heading: ['"Space Grotesk"', 'sans-serif'],
        body: ['Inter', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'monospace'],
      },
      boxShadow: {
        'glow-primary': '0 0 20px -5px rgba(0, 210, 255, 0.5)',
        'glow-primary-hover': '0 0 30px -5px rgba(0, 210, 255, 0.6)',
        'glow-accent': '0 0 20px rgba(58, 123, 213, 0.3)',
        'glow-focus': '0 10px 20px -10px rgba(128, 90, 215, 0.3)',
        'card-elevation': '0 0 50px -10px rgba(0, 210, 255, 0.1)',
        'card-hover': '0 0 30px -10px rgba(128, 90, 215, 0.2)',
        'card-active': '0 0 40px -10px rgba(0, 210, 255, 0.15)',
        'icon-hologram': '0 0 20px rgba(0, 210, 255, 0.4)',
      },
      backgroundImage: {
        'gradient-primary': 'linear-gradient(to right, #00D2FF, #3A7BD5, #805AD7, #FF42A1)',
        'gradient-accent': 'linear-gradient(to right, #805AD7, #FF42A1)',
        'glass': 'linear-gradient(to bottom right, rgba(255, 255, 255, 0.05), rgba(0, 0, 0, 0.4))',
      },
      animation: {
        'float': 'float 8s ease-in-out infinite',
        'spin-slow': 'spin 10s linear infinite',
        'spin-reverse': 'spin 15s linear infinite reverse',
        'ping-slow': 'ping 2s cubic-bezier(0, 0, 0.2, 1) infinite',
      },
      keyframes: {
        float: {
          '0%, 100%': { transform: 'translateY(0px)' },
          '50%': { transform: 'translateY(-20px)' },
        }
      }
    },
  },
  plugins: [],
};
