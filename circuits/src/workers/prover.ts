import { CircuitParams, type CircuitKind, type JobMessage, type WorkerInitArgs } from '../index.js'
import type { WorkerMessage } from './index.js'
import { plonk, type CircuitSignals } from 'snarkjs'
import { createFs } from 'indexeddb-fs'
import { blake3 } from '@noble/hashes/blake3'
import { hex } from '../utils.js'

const fs = createFs({ databaseName: 'zk-mahjong-circuits-fs' })

const readPtau = async (degree: number) => {
    const degreePadded = String(degree).padStart(2, '0')
    const filename = `${degreePadded}.ptau`
    return fs.readFile<Uint8Array>(filename)
}

const getR1cs = async (r1cs: string) => {
    const response = await fetch(r1cs)
    const data = await response.arrayBuffer()
    const r1csBin = new Uint8Array(data)
    const r1csHash = hex(blake3(r1csBin))
    const r1csHashFile = `${r1csHash}.r1cs`
    let outdated = true
    if (await fs.exists(r1csHashFile)) {
        const cachedR1cs = await fs.readFile<string>(r1csHashFile)
        outdated = cachedR1cs !== r1cs
    }
    console.log(`loaded ${r1cs} of hash ${r1csHash}, outdated: ${outdated}`)
    return {
        r1csBin,
        outdated
    }
}

const initZkey = async (circuit: CircuitKind, r1cs: string) => {
    const { r1csBin, outdated } = await getR1cs(r1cs)
    const zkeyExists = await fs.exists(`${circuit}.zkey`)
    const needBuild = !zkeyExists || outdated
    if (needBuild) {
        const { degree } = CircuitParams[circuit]
        const ptau = await readPtau(degree)
        const zkey = new Uint8Array()
        // @ts-expect-error Argument of type 'Uint8Array' is not assignable to parameter of type 'string'.
        await plonk.setup(r1csBin, ptau, zkey, console)
        console.log(`Built ${circuit}.zkey`, zkey)
        await fs.writeFile(`${circuit}.zkey`, zkey)
    }
}

let params: WorkerInitArgs | undefined

onmessage = ({
    data,
}: MessageEvent<WorkerMessage<WorkerInitArgs, JobMessage>>) => {
    if (data.kind === 'init') {
        init(data.args)
    } else if (data.kind === 'job') {
        const { circuit, input } = data.input
        handleJob(data.id, circuit, input)
    } else {
        console.warn(`Malformed message: ${JSON.stringify(data)}`)
    }
}

const init = async (args: WorkerInitArgs) => {
    params = args

    for (const circuit in params) {
        const { r1csPath } = params[circuit as CircuitKind]
        await initZkey(circuit as CircuitKind, r1csPath)
    }

    postMessage({ kind: 'init' })
}

const handleJob = async (
    id: number,
    circuit: CircuitKind,
    input: CircuitSignals
) => {
    try {
        const wasm = params![circuit].wasmPath
        const zk = await fs.readFile<Uint8Array>(`${circuit}.zkey`)
        const result = await plonk.fullProve(input, wasm, zk, console)
        postMessage({ kind: 'job', id, result })
    } catch (error) {
        postMessage({ kind: 'job', id, error })
    }
}
