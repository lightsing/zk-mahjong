use serde::{Serialize, Deserialize};

use crate::babyjubjub::{PublicKey, SecretKey, Point, BASE_POINT};
use crate::bn128::Fr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskedMessage {
    pub c0: Point,
    pub c1: Point,
}

impl MaskedMessage {
    pub fn new(m: Point, agg_pk: &PublicKey, randmomness: &Fr) -> MaskedMessage {
        let c0 = BASE_POINT.mul_scalar(&randmomness);
        let c1 = agg_pk.mul_scalar(&randmomness) + m;
        MaskedMessage {
            c0, c1: c1.affine()
        }
    }

    pub fn remask(&self, agg_pk: &PublicKey, randmomness: &Fr) -> MaskedMessage {
        let c0 = BASE_POINT.mul_scalar(&randmomness) + self.c0;
        let c1 = agg_pk.mul_scalar(&randmomness) + self.c1;
        MaskedMessage{
            c0: c0.affine(), c1: c1.affine()
        }
    }

    pub fn unmask(&self, sk: &SecretKey) -> MaskedMessage {
        MaskedMessage {
            c0: self.c0,
            c1: (self.c1 - self.c0.mul_scalar(&sk.0)).affine()
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::tile::{map::{TILES, TILE_MAP}, lookup_tile};

    use super::*;
    use ff::Field;


    #[test]
    fn test() {
        let sks: [SecretKey; 4] = std::array::from_fn(|_| SecretKey::random());
        let pks = sks.iter().map(|sk| sk.public_key()).collect::<Vec<_>>();
        let agg_pk = PublicKey::aggregate(pks);

        for idx in 0..144 {
            let m = TILES[idx].point;
            let randomness = Fr::random(rand::thread_rng());
            let masked = MaskedMessage::new(m, &agg_pk, &randomness);

            let unmasked = sks.iter().fold(masked, |acc, sk| acc.unmask(sk));
            assert_eq!(unmasked.c1, m);
            let tile = TILE_MAP.get(&unmasked.c1.x).unwrap();
            assert_eq!(tile, &TILES[idx]);
            let tile = lookup_tile(&unmasked.c1.x).unwrap();
            assert_eq!(tile, TILES[idx]);
            println!("{}: {:?}", idx, tile)
        }
    }
}