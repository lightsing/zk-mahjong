import { 
    genInitTileSet as genInitTileSetWasm,
} from 'zk-mahjong-wasm-sys'
import { MaskedMessage, type AggregatedMahjongPubkey, type FrString, type Point } from './key.ts'

export interface ShuffleResult {
    permutation: number[][],
    randomness: FrString[],
    tiles: MaskedMessage[]
}

export const genInitTileSet = (aggPk: AggregatedMahjongPubkey) => {
    const { permutation, randomness, tiles } =  genInitTileSetWasm(aggPk.key) as ShuffleResult

    return {
        permutation,
        randomness,
        tiles: tiles.map(t => new MaskedMessage(t))
    }
}