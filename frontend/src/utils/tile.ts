export enum TileKind {
    Man = 0,
    Pin = 1,
    Sou = 2,
    Zi = 3,
    Fa = 4,
}

export interface Tile {
    id: number
    kind: TileKind
    ord: number
    dup: number
}

export function getTileDisplay(tile: Tile): string {
    const { kind, ord } = tile
    switch (kind) {
        case TileKind.Man:
            return String.fromCodePoint(0x1f006 + ord)
        case TileKind.Pin:
            return String.fromCodePoint(0x1f018 + ord)
        case TileKind.Sou:
            return String.fromCodePoint(0x1f00f + ord)
        case TileKind.Zi:
            return String.fromCodePoint(0x1efff + ord)
        case TileKind.Fa:
            return String.fromCodePoint(0x1f021 + ord)
    }
}

export function idToTile(id: number): Tile {
    if (id >= 136) {
        return { id, kind: TileKind.Fa, ord: id - 136 + 1, dup: 0 }
    }
    const kind = Math.floor(id / 36) as TileKind
    let ord = Math.floor((id - 36 * kind) / 4) + 1
    const dup = id % 4
    if (kind === TileKind.Zi && ord >= 5) {
        // swap white dragon and red dragon
        ord = 12 - ord
    }
    return { id, kind, ord, dup }
}
