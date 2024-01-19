import { type PlonkProof, type PublicSignals } from 'snarkjs'
import ElGamalPubkeyProveWorker from './workers/elgamal_pubkey.ts?worker&inline'
import { WorkerDispatcher } from './workers/index.ts'

export interface FullProof {
    proof: PlonkProof
    publicSignals: PublicSignals
}

export interface ElgamalInitArgs {
    wasmPath: string
    zkeyPath: string
}

const elGamalPubkeyProveWorker = new WorkerDispatcher<ElgamalInitArgs, bigint, FullProof>(new ElGamalPubkeyProveWorker())

export const proveSecretKey = (sk: bigint) => elGamalPubkeyProveWorker.postMessage(sk)
export const initElgamalPubkeyProveWorker = (args: ElgamalInitArgs) => elGamalPubkeyProveWorker.init(args)