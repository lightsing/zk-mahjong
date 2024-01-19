import type { ElgamalInitArgs } from '../index.js'
import type { WorkerMessage } from './index.js'
import { plonk } from 'snarkjs'

let wasm: string | null = null
let zkey: string | null = null

onmessage = ({ data }: MessageEvent<WorkerMessage<ElgamalInitArgs, string>>) => {
    console.log(JSON.stringify(data))
    if (data.kind === 'init') {
        wasm = data.args.wasmPath
        zkey = data.args.zkeyPath
        postMessage({ kind: 'init' })
    } else if (data.kind === 'job') {
        handleJob(data.id, data.data, wasm!, zkey!)
    } else {
        console.warn(`Malformed message: ${JSON.stringify(data)}`)
    }
}

const handleJob = async (id: number, sk: string, wasm: string, zk: string) => {
    try {
        const skBig = BigInt(sk)
        const result = await plonk.fullProve({ skBig }, wasm, zk, console)
        postMessage({ kind: 'job', id, result })
    } catch (error) {
        postMessage({ kind: 'job', id, error })
    }
}