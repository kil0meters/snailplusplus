import type { Config } from 'tailwindcss';

const config: Config = {
    content: ['./src/**/*.{js,jsx,ts,tsx}'],
    theme: {
        extend: {
            colors: {
                'fg': '#068fef',
                'bg': '#110aef',
                'yellow': '#f8fc00'
            },
            zIndex: {
                'canvas': '10000000000000',
                'ui': '10000000000001'
            }
        },
    },
    plugins: [],
};

export default config;
