import { resolve, join } from 'path'
import { defineConfig } from 'vite'
import { globSync } from 'glob'
import checker from 'vite-plugin-checker'

export default defineConfig({
    appType: 'mpa',
    build: {
        rollupOptions: {
            input: [
                resolve(__dirname, 'index.html'),
                ...globSync(resolve(__dirname, 'pages/**/*.html')),
            ],
        },
        reportCompressedSize: false,
    },
    plugins: [checker({
        typescript: true,
    }),],
});
