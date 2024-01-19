import { resolve } from 'node:path'
import { defineConfig } from 'vite'
import { viteStaticCopy } from 'vite-plugin-static-copy'

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
    plugins: [
        viteStaticCopy({
            targets: [
                {
                    src: 'build/*/*/*.wasm',
                    dest: 'artifact'
                },
                {
                    src: 'build/*/*.zkey',
                    dest: 'artifact'
                },
                {
                    src: 'build/*/verification_key.json',
                    dest: 'artifact'
                }
            ]
        })
    ]
})
