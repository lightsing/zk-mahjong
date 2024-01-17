import { useEffect, useState } from 'react'
import './App.css'
import {
    MahjongKey,
    AggregatedMahjongPubkey,
    genInitTileSet,
    shuffleEncryptDeck,
} from 'zk-mahjong-wasm'
import TileComponent from './components/Tile'
import { idToTile } from './utils/tile'

function App() {
    const secretKeyList = Array.from({ length: 4 }).map((_) => {
        return new MahjongKey()
    })
    const publicKeyList = secretKeyList.map((e) => e.publicKey)

    const aggregatePublicKey = new AggregatedMahjongPubkey(publicKeyList)

    const initDeck = genInitTileSet()
    const { permutation, randomness, tiles } =
        shuffleEncryptDeck(aggregatePublicKey, initDeck)

    const decodedTiles = tiles
        .map((t) => secretKeyList[0].unmask(t))
        .map((t) => secretKeyList[1].unmask(t))
        .map((t) => secretKeyList[2].unmask(t))
        .map((t) => secretKeyList[3].unmask(t))
        .map((t) => t.tryReveal())
        .map((t) => idToTile(t as number))

    return (
        <>
            <div>
                {decodedTiles.map((tile) => {
                    return (
                        <span>
                            <TileComponent tile={tile} />
                        </span>
                    )
                })}
            </div>
        </>
    )
}

export default App
