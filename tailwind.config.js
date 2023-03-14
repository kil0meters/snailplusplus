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
            },
            animation: {
                "slide-out": "slideOut 1s linear forwards"
            },
            keyframes: {
                slideOut: {
                    '0%': {
                        transform: 'translateY(0)',
                        opacity: 1,
                    },
                    '50%': {
                        opacity: 1,
                    },
                    '100%': {
                        transform: 'translateY(-50px)',
                        opacity: 0,
                    }
                }
            }
        },
        fontFamily: {
            'pixelated': ['"Press Start 2P"', "monospace"],
            'display': ["Georgia", "serif"],
        }
    },
    plugins: [],
};
