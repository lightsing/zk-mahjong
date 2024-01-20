import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

import { zkMahjongInit } from 'zk-mahjong-wasm'

import { initProveWorker, type WorkerInitArgs } from '@zk-mahjong/circuits'

const config: WorkerInitArgs = {
    elGamalPubkey: {
        wasmPath: new URL(
            '@zk-mahjong/circuits/public_key/wasm',
            import.meta.url
        ).href,
        r1csPath: new URL(
            '@zk-mahjong/circuits/public_key/r1cs',
            import.meta.url
        ).href,
    },
    shuffleEncrypt: {
        wasmPath: new URL(
            '@zk-mahjong/circuits/shuffle_encrypt/wasm',
            import.meta.url
        ).href,
        r1csPath: new URL(
            '@zk-mahjong/circuits/shuffle_encrypt/r1cs',
            import.meta.url
        ).href,
    },
}
console.log(config)
Promise.all([
    zkMahjongInit(),
    initProveWorker(
        new Worker(new URL('@zk-mahjong/circuits/worker', import.meta.url)),
        config
    ),
]).then(() => {
    ReactDOM.createRoot(document.getElementById('root')!).render(<App />)
})
