import path from 'path'
import { fileURLToPath } from 'url'
import CopyWebpackPlugin from 'copy-webpack-plugin'
import HtmlWebpackPlugin from 'html-webpack-plugin'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

/** @type {import('webpack').Configuration} */
const config = {
    devtool: false,
    mode: process.env.NODE_ENV ?? 'development',
    entry: {
        main: './src/main.tsx',
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                loader: 'swc-loader',
                exclude: /node_modules/,
                options: {
                    jsc: {
                        preserveAllComments: true,
                        parser: {
                            syntax: 'typescript',
                            dynamicImport: true,
                            tsx: true,
                        },
                        target: 'es2022',
                        transform: {
                            react: {
                                runtime: 'automatic',
                                development: false,
                                refresh: false,
                            },
                        },
                    },
                },
            },
            {
                test: /\.css$/,
                use: ['style-loader', 'css-loader'],
            },
        ],
    },
    resolve: {
        extensions: ['.tsx', '.ts', '.js'],
    },
    output: {
        filename: 'bundle.js',
        clean: true,
    },
    plugins: [
        new CopyWebpackPlugin({
            patterns: [{ from: 'public' }],
        }),
        new HtmlWebpackPlugin({
            template: './index.html',
        }),
    ],
    experiments: {
        asyncWebAssembly: true,
        topLevelAwait: true,
    },
}

export default config
