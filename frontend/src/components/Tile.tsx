import './Tile.css'
import { Tile, getTileDisplay } from '../utils/tile'

interface TileProps {
    tile: Tile
}

export default function TileComponent({ tile }: TileProps) {
    return <span className="tile">{ getTileDisplay(tile) }</span>
}
