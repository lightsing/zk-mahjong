import type { CircuitKind, JobMessage, WorkerInitArgs } from '../index.js'
import type { WorkerMessage } from './index.js'
import { plonk, type CircuitSignals } from 'snarkjs'

let params: WorkerInitArgs | null = null

onmessage = ({
    data,
}: MessageEvent<WorkerMessage<WorkerInitArgs, JobMessage>>) => {
    if (data.kind === 'init') {
        params = data.args
        postMessage({ kind: 'init' })
    } else if (data.kind === 'job') {
        const { circuit, input } = data.input
        handleJob(data.id, circuit, input)
    } else {
        console.warn(`Malformed message: ${JSON.stringify(data)}`)
    }
}

const handleJob = async (
    id: number,
    circuit: CircuitKind,
    input: CircuitSignals
) => {
    try {
        const wasm = params![circuit].wasmPath
        const zk = params![circuit].zkeyPath
        const result = await plonk.fullProve(input, wasm, zk, console)
        postMessage({ kind: 'job', id, result })
    } catch (error) {
        postMessage({ kind: 'job', id, error })
    }
}
