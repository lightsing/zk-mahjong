use halo2curves::{
    bn256::Fq,
    group::{Curve, Group, GroupEncoding},
    grumpkin::{G1Affine, G1},
};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskedMessage {
    pub c0: G1,
    pub c1: G1,
}

impl MaskedMessage {
    pub fn new(m: G1) -> MaskedMessage {
        MaskedMessage {
            c0: G1::identity(),
            c1: m,
        }
    }

    pub fn remask(&self, agg_pk: &G1, randomness: &Fq) -> MaskedMessage {
        let c0 = G1::generator() * randomness + self.c0;
        let c1 = agg_pk * randomness + self.c1;
        MaskedMessage { c0, c1 }
    }

    pub fn unmask(&self, sk: &Fq) -> MaskedMessage {
        MaskedMessage {
            c0: self.c0,
            c1: self.c1 - self.c0 * sk,
        }
    }

    pub fn get_message(&self) -> G1Affine {
        self.c1.to_affine()
    }

    pub fn compress(&self) -> [u8; 64] {
        let c0 = self.c0.to_bytes();
        let c1 = self.c1.to_bytes();
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(c0.as_ref());
        buf[32..].copy_from_slice(c1.as_ref());

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{
        map::{TILES, TILE_MAP},
        BaseTile,
    };
    use ff::Field;
    use halo2curves::group::cofactor::CofactorCurveAffine;
    use rand::thread_rng;

    #[test]
    fn test() {
        let sks: [Fq; 4] = std::array::from_fn(|_| Fq::random(thread_rng()));
        let pks = sks
            .iter()
            .map(|sk| G1::generator() * sk)
            .collect::<Vec<_>>();
        let agg_pk = pks.iter().fold(G1::identity(), |acc, pk| acc + pk);

        for idx in 0..144 {
            let m = MaskedMessage::new(TILES[idx].point.to_curve());
            let randomness = Fq::random(thread_rng());
            let masked = m.remask(&agg_pk, &randomness);

            let unmasked_g1 = sks.iter().fold(masked, |acc, sk| acc.unmask(sk));
            let unmasked_g1_affine = unmasked_g1.c1.to_affine();
            assert_eq!(unmasked_g1_affine, m.c1.to_affine(), "{}", idx);
            assert_eq!(unmasked_g1_affine, TILES[idx].point, "{}", idx);
            let tile = TILE_MAP.get(&unmasked_g1_affine.x.to_bytes()).unwrap();
            assert_eq!(tile, &TILES[idx]);
            let tile = BaseTile::lookup(&unmasked_g1_affine.x).unwrap();
            assert_eq!(tile, TILES[idx]);
        }
    }

    #[test]
    fn test_serialization() {
        let m = MaskedMessage::new(G1::generator());
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(
            json,
            r#"{"c0":"0000000000000000000000000000000000000000000000000000000000000080","c1":"0100000000000000000000000000000000000000000000000000000000000000"}"#
        )
    }
}
