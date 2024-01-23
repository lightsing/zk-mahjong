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
use std::ops::{Deref, DerefMut};

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

impl BaseTile {
    pub fn lookup(x: &Fr) -> Option<Self> {
        TILE_MAP.get(&x.to_bytes()).copied()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilesDeck<const N: usize> {
    #[serde(with = "serde_big_array::BigArray")]
    pub tiles: [MaskedMessage; N],
}

impl TilesDeck<136> {
    pub fn richi() -> Self {
        let tiles = TILES[0..136]
            .iter()
            .map(|t| MaskedMessage::new(t.point.to_curve()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Self { tiles }
    }
}

impl TilesDeck<144> {
    pub fn full() -> Self {
        Self {
            tiles: TILES.map(|t| MaskedMessage::new(t.point.to_curve())),
        }
    }
}

impl<const N: usize> TilesDeck<N> {
    fn gen_randomness() -> [Fq; N] {
        let mut rng = rand::thread_rng();
        [(); N].map(|_| Fq::from_bytes(&Fr::random(&mut rng).to_bytes()).unwrap())
    }

    pub fn shuffle_encrypt(&self, agg_pk: &G1) -> ShuffleEncryptResult<N> {
        let randomness = Self::gen_randomness();
        let mut tiles = self
            .tiles
            .into_iter()
            .zip(randomness.iter())
            .map(|(tile, randomness)| tile.remask(agg_pk, randomness))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let permutation = PermutationMatrix::new();
        permutation.apply(&mut tiles);
        ShuffleEncryptResult {
            randomness,
            tiles,
            permutation,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermutationMatrix<const N: usize>(
    #[serde(with = "serde_big_array::BigArray")] [MatrixRow<N>; N],
);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MatrixRow<const N: usize>(#[serde(with = "serde_big_array::BigArray")] [u8; N]);

impl<const N: usize> Deref for MatrixRow<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for MatrixRow<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> Default for PermutationMatrix<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PermutationMatrix<N> {
    #[allow(clippy::needless_range_loop)]
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut matrix = [MatrixRow([0; N]); N];
        for i in 0..N {
            matrix[i][i] = 1;
        }
        for i in 0..N {
            let j = rng.gen_range(0..N);
            matrix.swap(i, j);
        }
        Self(matrix)
    }

    /// Apply the permutation matrix to a vector.
    pub fn apply(&self, v: &mut [MaskedMessage; N]) {
        let temp = v.to_vec();
        for (i, row) in self.0.iter().enumerate() {
            v[i] = temp[row.iter().position(|&x| x == 1).unwrap()];
        }
    }

    /// Compress the permutation matrix into a vector of field elements.
    ///
    /// The vector is compressed by representing each row as a power of 2.
    ///
    /// For example, the identity matrix is represented as:
    /// `[1, 2, 4, 8, 16, 32, 64, 128, 256 ... 2^N-1]`
    pub fn compress(&self) -> [Fr; N] {
        const F2: Fr = Fr::from_raw([2, 0, 0, 0]);
        let mut buf = [Fr::ZERO; N];
        for (i, row) in self.0.iter().enumerate() {
            let pos = row.iter().position(|&x| x == 1).unwrap() as u64;
            buf[i] = F2.pow_vartime([pos, 0, 0, 0])
        }
        buf
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ShuffleEncryptResult<const N: usize> {
    #[serde(with = "serde_big_array::BigArray")]
    pub randomness: [Fq; N],
    #[serde(with = "serde_big_array::BigArray")]
    pub tiles: [MaskedMessage; N],
    pub permutation: PermutationMatrix<N>,
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
    writeln!(&mut file, ";").unwrap();
}
