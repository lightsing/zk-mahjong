import {
    type CircuitSignals,
    type PlonkProof,
    type PublicSignals,
} from 'snarkjs'
import { WorkerDispatcher } from './workers/index.js'

export type CircuitKind = 'elGamalSecretKey'

export interface FullProof {
    proof: PlonkProof
    publicSignals: PublicSignals
}

export interface InitArgs {
    wasmPath: string
    zkeyPath: string
}

export type WorkerInitArgs = {
    [circuit in CircuitKind]: InitArgs
}

export interface JobMessage {
    circuit: CircuitKind
    input: CircuitSignals
}

let proverWorker: WorkerDispatcher<
    WorkerInitArgs,
    JobMessage,
    FullProof
> | null = null

export const initProveWorker = async (worker: Worker, args: WorkerInitArgs) => {
    if (proverWorker !== null) {
        throw new Error('ElGamal Pubkey Prove Worker already set')
    }
    proverWorker = new WorkerDispatcher(worker)
    await proverWorker.init(args)
}

export const proveSecretKey = (sk: bigint) => {
    if (proverWorker === null) {
        throw new Error('ElGamal Pubkey Prove Worker not set')
    }
    return proverWorker.postMessage({
        circuit: 'elGamalSecretKey',
        input: { sk: sk.toString() },
    })
}
