import { type PlonkProof, type PublicSignals } from 'snarkjs'
import { WorkerDispatcher } from './workers/index.js'

export interface FullProof {
    proof: PlonkProof
    publicSignals: PublicSignals
}

export interface ElgamalInitArgs {
    wasmPath: string
    zkeyPath: string
}

let elGamalPubkeyProveWorker: WorkerDispatcher<ElgamalInitArgs, string, FullProof> | null = null

export const initElGamalPubkeyProveWorker = async (worker: Worker, args: ElgamalInitArgs) => {
    if (elGamalPubkeyProveWorker !== null) {
        throw new Error('ElGamal Pubkey Prove Worker already set')
    }
    elGamalPubkeyProveWorker = new WorkerDispatcher(worker)
    await elGamalPubkeyProveWorker.init(args)
}

export const proveSecretKey = (sk: bigint) => {
    if (elGamalPubkeyProveWorker === null) {
        throw new Error('ElGamal Pubkey Prove Worker not set')
    }
    return elGamalPubkeyProveWorker.postMessage(sk.toString())
}