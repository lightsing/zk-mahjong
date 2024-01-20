import typescript from '@rollup/plugin-typescript'
import { nodeResolve } from '@rollup/plugin-node-resolve'

export default [
    {
        input: './src/index.ts',
        output: {
            file: './dist/main.js',
            format: 'esm',
            sourcemap: true,
        },
        plugins: [typescript(), nodeResolve({ browser: true })],
    },
    {
        input: './src/workers/prover.ts',
        output: {
            file: './dist/prover.worker.js',
            format: 'esm',
            sourcemap: true,
        },
        plugins: [typescript(), nodeResolve({ browser: true })],
    },
]
