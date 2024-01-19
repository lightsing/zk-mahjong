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
import { proveSecretKey, initElgamalPubkeyProveWorker } from '@zk-mahjong/circuits'
import ElGamalPubkeyWasm from '../../circuits/build/elgamal_pubkey/elgamal_pubkey_js/elgamal_pubkey.wasm?url'
import ElGamalPubkeyZkey from '../../circuits/build/elgamal_pubkey/elgamal_pubkey.zkey?url'

function App() {
    const secretKeyList = Array.from({ length: 4 }).map((_) => {
        return new MahjongKey()
    })
    const publicKeyList = secretKeyList.map((e) => e.publicKey)

    useEffect(() => {
        console.log(secretKeyList)
        console.log(ElGamalPubkeyWasm)
        const prove = async () => {
            await initElgamalPubkeyProveWorker({
                wasmPath: ElGamalPubkeyWasm,
                zkeyPath: ElGamalPubkeyZkey
            })
            // const {proof, publicSignals} = await proveSecretKey(secretKeyList[0].toBigInt())
            // console.log(proof, publicSignals)
            // console.assert(secretKeyList[0].publicKey.key.x === publicSignals[0])
            // console.assert(secretKeyList[0].publicKey.key.y === publicSignals[1])
        }
        prove()
    }, [])

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
