import { useEffect } from 'react'
import './App.css'
import {
    MahjongKey,
    AggregatedMahjongPubkey,
    genInitTileSet,
    shuffleEncryptDeck,
} from 'zk-mahjong-wasm'
import TileComponent from './components/Tile'
import { idToTile } from './utils/tile'
import { proveSecretKey } from '@zk-mahjong/circuits'

function App() {
    const secretKeyList = Array.from({ length: 4 }).map((_) => {
        return new MahjongKey()
    })
    const publicKeyList = secretKeyList.map((e) => e.publicKey)

    useEffect(() => {
        console.log(secretKeyList)
        const prove = async () => {
            const {proof, publicSignals} = await proveSecretKey(secretKeyList[0].toBigInt())
            console.log(proof, publicSignals)
            console.assert(secretKeyList[0].publicKey.key.x === publicSignals[0])
            console.assert(secretKeyList[0].publicKey.key.y === publicSignals[1])
        }
        prove()
    }, [])

    const aggregatePublicKey = new AggregatedMahjongPubkey(publicKeyList)

    const initDeck = genInitTileSet()
    const { tiles } =
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
