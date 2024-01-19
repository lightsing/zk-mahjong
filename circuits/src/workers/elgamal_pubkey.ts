import type { ElgamalInitArgs } from '../index.ts'
import type { WorkerMessage } from './index.ts'
import { plonk } from 'snarkjs'

let elGamalPubkeyWitnessGen: string | undefined
let elGamalPubkeyZkey: string | undefined

onmessage = ({ data }: MessageEvent<WorkerMessage<ElgamalInitArgs, bigint>>) => {
    if (data.kind === 'init') {
        elGamalPubkeyWitnessGen = data.args.wasmPath
        elGamalPubkeyZkey = data.args.zkeyPath
        postMessage({ kind: 'init' })
    } else if (data.kind === 'job') {
        handleJob(data.id, data.data)
    } else {
        throw new Error(`Malformed message: ${data}`)
    }
}

const handleJob = async (id: number, sk: bigint) => {
    try {
        const result = plonk.fullProve({ sk }, elGamalPubkeyWitnessGen!, elGamalPubkeyZkey!)
        postMessage({ kind: 'job', id, result })
    } catch (error) {
        postMessage({ kind: 'job', id, error })
    }
}