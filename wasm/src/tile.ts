import {
    genInitTileSet as genInitTileSetWasm,
    shuffleEncryptDeck as shuffleEncryptDeckWasm,
} from 'zk-mahjong-wasm-sys'
import {
    MaskedMessage,
    type AggregatedMahjongPubkey,
    type FrString,
    type Point,
} from './key.js'

export interface ShuffleResult {
    permutation: number[][]
    randomness: FrString[]
    tiles: MaskedMessage[]
}

export const genInitTileSet = () => genInitTileSetWasm() as MaskedMessage[]

export const shuffleEncryptDeck = (
    agg_pk: AggregatedMahjongPubkey,
    tiles: MaskedMessage[]
) => {
    const result = shuffleEncryptDeckWasm(agg_pk.key, tiles)

    return {
        permutation: result.permutation,
        randomness: result.randomness,
        tiles: result.tiles.map(
            (tile: { c0: Point; c1: Point }) => new MaskedMessage(tile)
        ),
    } as ShuffleResult
}
