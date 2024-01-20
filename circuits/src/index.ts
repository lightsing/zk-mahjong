import {
    type CircuitSignals,
    type PlonkProof,
    type PublicSignals,
} from 'snarkjs'
import { WorkerDispatcher } from './workers/index.js'
import { createFs } from 'indexeddb-fs';
import { sha256 } from '@noble/hashes/sha256';
import { hex } from './utils.js';

export type CircuitKind = 'elGamalPubkey' | 'shuffleEncrypt'

export const CircuitParams = {
    elGamalPubkey: {
        degree: 8,
    },
    shuffleEncrypt: {
        degree: 21,
    },
}

export interface FullProof {
    proof: PlonkProof
    publicSignals: PublicSignals
}

export interface InitArgs {
    wasmPath: string,
    r1csPath: string,
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

const fs = createFs({ databaseName: 'zk-mahjong-circuits-fs' })

const ptaus: Record<number, {
    QmHash: string
    sha256: string
}> = {
    8: {
        QmHash: 'QmNwT4UN6gT7vdDPNjmpShEVVbhi6C7tR6Y98X4aCT7sbq',
        sha256: 'f741f2ddee2875915c24db8aae90d021f51181533f1ee3b58baf64b042e91654'
    },
    21: {
        QmHash: 'QmQtuQMercz2WHgdzoMgKg7DXka8JEwY6fvNU2L5qzbKQU',
        sha256: 'cdc7c94a6635bc91466d8c7d96faefe1d17ecc98a3596a748ca1e6c895f8c2b4'
    },
}

const ensurePtau = async (degree: number) => {
    const degreePadded = String(degree).padStart(2, '0')
    const filename = `${degreePadded}.ptau`
    if (await fs.exists(filename)) {
        const ptau = await fs.readFile<Uint8Array>(filename)
        if (hex(sha256(ptau)) === ptaus[degree].sha256) {
            return
        }
        console.warn(`Ptau hash mismatch, redownloading from IPFS`)
    }
    const url = `https://cloudflare-ipfs.com/ipfs/${ptaus[degree].QmHash}`
    console.log(`Ptau not found in cache, downloading from ${url}`)
    const response = await fetch(url)
    const data = await response.arrayBuffer()
    const ptau = new Uint8Array(data)
    await fs.writeFile(filename, ptau).catch(console.error)
}

export const initProveWorker = async (worker: Worker, args: WorkerInitArgs) => {
    if (proverWorker !== null) {
        throw new Error('ElGamal Pubkey Prove Worker already set')
    }
    const degreeInuse = new Set(Object.values(CircuitParams).map(c => c.degree))
    for (const degree of degreeInuse) {
        await ensurePtau(degree)
    }
    proverWorker = new WorkerDispatcher(worker)
    await proverWorker.init(args)
}

export const proveSecretKey = (sk: bigint) => {
    if (proverWorker === null) {
        throw new Error('ElGamal Pubkey Prove Worker not set')
    }
    return proverWorker.postMessage({
        circuit: 'elGamalPubkey',
        input: { sk: sk.toString() },
    })
}
