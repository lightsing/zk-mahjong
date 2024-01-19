import typescript from '@rollup/plugin-typescript';
import { nodeResolve } from '@rollup/plugin-node-resolve';

export default [
    {
        input: './src/index.ts',
        output: {
            file: './dist/main.js',
            format: 'esm',
        },
        plugins: [typescript(), nodeResolve({browser: true})],
    },
    {
        input: './src/workers/elgamal_pubkey.ts',
        output: {
            file: './dist/elgamal_pubkey.worker.js',
            format: 'esm',
        },
        plugins: [typescript(), nodeResolve({browser: true})],
    }
]