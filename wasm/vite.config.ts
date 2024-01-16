import { resolve } from 'node:path'
import { defineConfig } from 'vite'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'

// https://vitejs.dev/config/
export default defineConfig({
    build: {
        lib: {
            entry: resolve(__dirname, 'src/index.ts'),
            formats: ['es'],
        },
        sourcemap: true,
        emptyOutDir: false,
        minify: false,
    },
    plugins: [wasm(), topLevelAwait()],
})
