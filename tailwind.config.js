/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./templates/**/*.html"],
  theme: {
    extend: {
      colors: {
        star: {
          lighter: '#f6eabd',
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
      'intro': 'intro 1s forwards',
    },
    keyframes: {
      intro: {
        '0%': { 'opacity': '0' },
        '50%': { 'opacity': '0' },
        '100%': { 'opacity': '1' },
      },
      slidein: {
        '0%': { 'margin-top': '-10vh' },
        '100%': { 'margin-top': '10vh' },
      }
    }
  },
  plugins: [],
}
