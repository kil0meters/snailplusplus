/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        './index.html',
        './src/**/*.{js,ts,jsx,tsx,css,md,mdx,html,json,scss}',
    ],
    darkMode: 'class',
    theme: {
        extend: {
            boxShadow: {
                'blue': '0 0px 10px 0px rgba(0, 0, 255, 0.5)'
            }
        },
        fontFamily: {
            'pixelated': ['"Press Start 2P"', "monospace"],
            'display': ["Georgia", "serif"],
        }
    },
    plugins: [],
};
