use self::map::{TILES, TILE_MAP};
use crate::elgamal::MaskedMessage;
use ff::Field;
use halo2curves::{
    bn256::{Fq, Fr},
    group::cofactor::CofactorCurveAffine,
    grumpkin::{G1Affine, G1},
};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[rustfmt::skip]
pub(crate) mod map;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileKind {
    Man,
    Pin,
    Sou,
    Zi,
    Fa,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseTile {
    pub idx: usize,
    pub point: G1Affine,
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
    pub randomness: Vec<Fq>,
    pub tiles: Vec<MaskedMessage>,
    pub permutation: PermutationMatrix,
}

pub fn gen_randomness(n: usize) -> Vec<Fq> {
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| Fq::from_bytes(&Fr::random(&mut rng).to_bytes()).unwrap())
        .collect()
}

pub fn get_richi_tiles() -> Vec<MaskedMessage> {
    TILES[0..136]
        .iter()
        .map(|t| MaskedMessage::new(t.point.to_curve()))
        .collect()
}

pub fn get_full_tiles() -> Vec<MaskedMessage> {
    TILES
        .iter()
        .map(|t| MaskedMessage::new(t.point.to_curve()))
        .collect()
}

pub fn shuffle_encrypt_deck(agg_pk: &G1, tiles: &[MaskedMessage]) -> ShuffleEncryptResult {
    let randomness = gen_randomness(136);
    let mut tiles: Vec<MaskedMessage> = tiles
        .iter()
        .zip(randomness.iter())
        .map(|(tile, randomness)| tile.remask(agg_pk, randomness))
        .collect();
    let permutation = PermutationMatrix::new(136);
    permutation.apply(&mut tiles);
    ShuffleEncryptResult {
        randomness,
        tiles,
        permutation,
    }
}

pub fn lookup_tile(x: &Fr) -> Option<BaseTile> {
    TILE_MAP.get(&x.to_bytes()).copied()
}

#[test]
#[ignore]
fn gen_tile_map() {
    use halo2curves::bn256::Fq;
    use halo2curves::group::Curve;
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create("src/tile/map.rs").unwrap();
    writeln!(file, "use halo2curves::grumpkin::G1Affine;").unwrap();
    writeln!(file, "use halo2curves::bn256::Fr;").unwrap();
    writeln!(file, "use crate::tile::*;\n").unwrap();
    writeln!(file, "pub const TILES: [BaseTile; 144] = [").unwrap();
    let tiles: [G1Affine; 144] =
        std::array::from_fn(|i| (G1Affine::generator() * Fq::from(i as u64 + 2)).to_affine());
    let literal_fn = |n: Fr| {
        let mut buf = String::from("Fr::from_raw([");
        let mut bytes = n.to_bytes();
        bytes.reverse();
        let hex = hex::encode(bytes);
        for i in 0..4 {
            buf.push_str("0x");
            buf.push_str(&hex[64 - (i + 1) * 16..64 - i * 16]);
            if i != 3 {
                buf.push_str(", ");
            }
        }
        buf.push_str("])");
        buf
    };
    let literals: [(String, String); 144] = tiles.map(|p| (literal_fn(p.x), literal_fn(p.y)));

    for (i, kind) in [TileKind::Man, TileKind::Pin, TileKind::Sou]
        .into_iter()
        .enumerate()
    {
        for ord in 0..9 {
            for dup in 0..4 {
                let idx = i * 9 * 4 + ord * 4 + dup;
                writeln!(file, "    BaseTile {{ idx: {idx}, point: G1Affine {{ x: {}, y: {} }}, kind: TileKind::{:?}, ord: {} }},", literals[idx].0, literals[idx].1, kind, ord + 1).unwrap();
            }
        }
    }
    for ord in 0..7 {
        for dup in 0..4 {
            let idx = 3 * 9 * 4 + ord * 4 + dup;
            writeln!(file, "    BaseTile {{ idx: {idx}, point: G1Affine {{ x: {}, y: {} }}, kind: TileKind::Zi, ord: {} }},", literals[idx].0, literals[idx].1, ord + 1).unwrap();
        }
    }
    for ord in 0..8 {
        let idx = 3 * 9 * 4 + 7 * 4 + ord;
        writeln!(file, "    BaseTile {{ idx: {idx}, point: G1Affine {{ x: {}, y: {} }}, kind: TileKind::Fa, ord: {} }},", literals[idx].0, literals[idx].1, ord + 1).unwrap();
    }
    writeln!(file, "];\n").unwrap();

    let mut map = phf_codegen::Map::new();
    for (idx, tile) in tiles.iter().enumerate() {
        map.entry(tile.x.to_bytes(), &format!("TILES[{}]", idx));
    }

    write!(
        &mut file,
        "pub static TILE_MAP: phf::Map<[u8; 32], BaseTile> = {}",
        map.build()
    )
    .unwrap();
    write!(&mut file, ";\n").unwrap();
}
