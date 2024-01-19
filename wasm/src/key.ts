import {
    zkMahjongKeyGen,
    zkMahjongKeyToPubkey,
    zkMahjongPubkeyAggregate,
    zkMahjongUnmaskMessage,
    lookupTile,
} from 'zk-mahjong-wasm-sys'

export type FrString = string

export interface Point {
    x: FrString
    y: FrString
}

export class MaskedMessage {
    readonly c0: Point
    readonly c1: Point

    constructor({ c0, c1 }: { c0: Point; c1: Point }) {
        this.c0 = c0
        this.c1 = c1
    }

    tryReveal(): number | null {
        return lookupTile(this.c1.x)
    }
}

export class AggregatedMahjongPubkey {
    readonly key: Point

    constructor(pubkeys: MahjongPubkey[]) {
        this.key = zkMahjongPubkeyAggregate(
            pubkeys.map((pubkey) => pubkey.key) as Point[]
        )
    }
}

export class MahjongPubkey {
    readonly key: Point

    private constructor(key: Point) {
        this.key = key
    }

    static fromPrivateKey(key: FrString): MahjongPubkey {
        return new MahjongPubkey(zkMahjongKeyToPubkey(key) as Point)
    }
}

export class MahjongKey {
    private readonly key: FrString
    readonly publicKey: MahjongPubkey

    constructor() {
        this.key = zkMahjongKeyGen()
        this.publicKey = MahjongPubkey.fromPrivateKey(this.key)
    }

    toBigInt(): bigint {
        return BigInt(this.key)
    }

    unmask(maskedMessage: MaskedMessage): MaskedMessage {
        const c1 = zkMahjongUnmaskMessage(this.key, maskedMessage)
        return new MaskedMessage({
            c0: maskedMessage.c0,
            c1: c1 as Point,
        })
    }
}
