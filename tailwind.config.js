/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./templates/**/*.html"],
  theme: {
    extend: {
      colors: {
        star: {
          DEFAULT: '#ffd700',
          dark: '#fdbb0b',
          darker: '#92400e'
        },
        offwhite: '#f1f5f9',
        offwhite2: '#f2f2f2',
        black: '#0e1116',
        offblack: '#171b21',
        offblack2: '#252b32',
      }
    },
    animation: {
      'slidein': 'slidein .3s',
    },
    keyframes: {
      slidein: {
        '0%': { 'margin-top': '-10vh' },
        '100%': { 'margin-top': '10vh' },
      }
    }
  },
  plugins: [],
}
