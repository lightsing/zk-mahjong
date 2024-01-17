use rand::Rng;
use serde::Serialize;
use crate::{babyjubjub::{Point, PublicKey}, bn128::Fr, elgamal::MaskedMessage};
use ff::Field;
use self::map::{TILE_MAP, TILES};

#[rustfmt::skip]
pub(crate) mod map;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TileKind {
    Man,
    Pin,
    Sou,
    Zi,
    Fa
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BaseTile {
    pub idx: usize,
    pub point: Point,
    pub kind: TileKind,
    pub ord: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PermutationMatrix(Vec<Vec<u8>>);

impl PermutationMatrix {
    pub fn new(n: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut matrix = vec![vec![0; n]; n];
        for i in 0..n {
            matrix[i][i] = 1;
        }
        for i in 0..n {
            let j = rng.gen_range(0..n);
            matrix.swap(i, j);
        }
        Self(matrix)
    }

    /// Apply the permutation matrix to a vector.
    pub fn apply(&self, v: &mut [MaskedMessage]) {
        assert_eq!(self.0.len(), v.len());
        let temp = v.to_vec();
        for (i, row) in self.0.iter().enumerate() {
            v[i] = temp[row.iter().position(|&x| x == 1).unwrap()];
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ShuffleEncryptResult {
    pub randomness: Vec<Fr>,
    pub tiles: Vec<MaskedMessage>,
    pub permutation: PermutationMatrix,
}

pub fn gen_randomness(n: usize) -> Vec<Fr> {
    let mut rng = rand::thread_rng();
    (0..n).map(|_| Fr::random(&mut rng)).collect()
}

pub fn get_richi_tiles() -> Vec<MaskedMessage> {
    TILES[0..136].iter().map(|t| MaskedMessage::new(t.point)).collect()
}

pub fn get_full_tiles() -> Vec<MaskedMessage> {
    TILES.iter().map(|t| MaskedMessage::new(t.point)).collect()
}

pub fn shuffle_encrypt_deck(
    agg_pk: &PublicKey,
    tiles: &[MaskedMessage],
) -> ShuffleEncryptResult {
    let randomness = gen_randomness(136);
    let mut tiles: Vec<MaskedMessage> = tiles
        .iter()
        .zip(randomness.iter())
        .map(|(tile, randomness)| tile.remask(agg_pk, randomness))
        .collect();
    let permutation = PermutationMatrix::new(136);
    permutation.apply(&mut tiles);
    ShuffleEncryptResult { randomness, tiles, permutation }
}

pub fn lookup_tile(x: &Fr) -> Option<BaseTile> {
    info!("lookup tile: {}", x);
    TILE_MAP.get(x).copied()
}

#[test]
#[ignore]
fn gen_tile_map() {
    use std::fs::File;
    use std::io::Write;
    use crate::babyjubjub::BASE_POINT;

    let mut file = File::create("src/tile/map.rs").unwrap();
    writeln!(file, "use crate::babyjubjub::Point;").unwrap();
    writeln!(file, "use crate::bn128::Fr;").unwrap();
    writeln!(file, "use crate::tile::*;\n").unwrap();
    writeln!(file, "pub const TILES: [BaseTile; 144] = [").unwrap();
    let tiles: [Point; 144] = std::array::from_fn(|i| BASE_POINT.mul_scalar(&Fr::from(i as u64 + 1)));
    let literal_fn = |n: Fr| {
        let mut buf = String::from("Fr::from_raw([");
        let raw = n.into_raw();
        for i in 0..4 {
            buf.push_str(&format!("0x{:016x}, ", raw[i]));
        }
        buf.push_str("])");
        buf
    };
    let literals: [(String, String); 144] = tiles.map(|p| {
        (literal_fn(p.x), literal_fn(p.y))
    });
    
    for (i, kind) in [
        TileKind::Man,
        TileKind::Pin,
        TileKind::Sou,
    ].into_iter().enumerate() {
        for ord in 0..9 {
            for dup in 0..4 {
                let idx = i * 9 * 4 + ord * 4 + dup;
                writeln!(file, "    BaseTile {{ idx: {idx}, point: Point {{ x: {}, y: {} }}, kind: TileKind::{:?}, ord: {} }},", literals[idx].0, literals[idx].1, kind, ord + 1).unwrap();
            }
        }
    }
    for ord in 0..7 {
        for dup in 0..4 {
            let idx = 3 * 9 * 4 + ord * 4 + dup;
            writeln!(file, "    BaseTile {{ idx: {idx}, point: Point {{ x: {}, y: {} }}, kind: TileKind::Zi, ord: {} }},", literals[idx].0, literals[idx].1, ord + 1).unwrap();
        }
    }
    for ord in 0..8 {
        let idx = 3 * 9 * 4 + 7 * 4 + ord;
        writeln!(file, "    BaseTile {{ idx: {idx}, point: Point {{ x: {}, y: {} }}, kind: TileKind::Fa, ord: {} }},", literals[idx].0, literals[idx].1, ord + 1).unwrap();
    }
    writeln!(file, "];\n").unwrap();

    let mut map = phf_codegen::Map::new();
    for (idx, tile) in tiles.iter().enumerate() {
        map.entry(tile.x, &format!("TILES[{}]", idx));
    }

    write!(
        &mut file,
        "pub static TILE_MAP: phf::Map<Fr, BaseTile> = {}",
        map.build()
    )
    .unwrap();
    write!(&mut file, ";\n").unwrap();
}