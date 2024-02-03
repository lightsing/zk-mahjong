use crate::utils::assume;
use ff::{derive::subtle::CtOption, PrimeField};
use halo2curves::goldilocks::Fp;
use itertools::Itertools;
use std::ops::{AddAssign, Mul};
use std::{array, iter};

/// Split a 256-bit integer into 11 limbs.
/// Each limb is 24 bits except for the last one, which is 16 bits.
///
/// The field repr should be little-endian.
pub fn split_into_limbs<F: PrimeField<Repr = [u8; 32]>, L: From<u64>>(x: F) -> [L; 11] {
    let repr = x.to_repr();
    array::from_fn(|i| {
        if i < 10 {
            L::from(
                u64::from(repr[3 * i])
                    | (u64::from(repr[3 * i + 1]) << 8)
                    | (u64::from(repr[3 * i + 2]) << 16),
            )
        } else {
            L::from(u64::from(repr[3 * i]) | (u64::from(repr[3 * i + 1]) << 8))
        }
    })
}

pub fn fr_from_limbs<F: PrimeField<Repr = [u8; 32]>>(limbs: &[Fp; 11]) -> Option<F> {
    let mut repr = [0; 32];
    for (i, limb) in limbs.iter().enumerate() {
        let limb = limb.to_canonical_u64_vartime();
        assume(limb < (1 << 24));
        repr[3 * i] = limb as u8;
        repr[3 * i + 1] = (limb >> 8) as u8;
        if i < 10 {
            repr[3 * i + 2] = (limb >> 16) as u8;
        }
    }
    F::from_repr_vartime(repr)
}

pub fn limbs_product(a_limbs: &[u64; 11], b_limbs: &[u64; 11]) -> [u64; 22] {
    let mut prod_limbs = [0; 22];
    for (i, a_limb) in a_limbs.iter().enumerate() {
        for (j, b_limb) in b_limbs.iter().enumerate() {
            prod_limbs[i + j] += a_limb * b_limb;
        }
    }
    #[cfg(debug_assertions)]
    {
        let modulus = 0xffffffff00000001u64;
        for limb in prod_limbs.iter() {
            assert!(*limb < modulus);
        }
    }
    prod_limbs
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff::Field;
    use halo2curves::goldilocks::Fp;
    use halo2curves::secp256k1::Fp as Fr;
    use num_bigint::BigUint;
    use rand::thread_rng;

    #[test]
    fn test() {
        let modulus = Fr::ZERO - Fr::ONE;
        let mut modulus_limbs = split_into_limbs::<_, u64>(modulus);
        modulus_limbs[0] += 1;
        let modulus_big = BigUint::from_bytes_le(modulus.to_repr().as_ref()) + BigUint::from(1u64);

        let a = Fr::random(thread_rng());
        let a_big = BigUint::from_bytes_le(a.to_repr().as_ref());
        let a_limbs = split_into_limbs::<_, u64>(a);
        let b = Fr::random(thread_rng());
        let b_big = BigUint::from_bytes_le(b.to_repr().as_ref());
        let b_limbs = split_into_limbs::<_, u64>(b);

        // a * b - k * p = c
        let c = a * b;
        let c_big = BigUint::from_bytes_le(c.to_repr().as_ref());
        let c_limbs = split_into_limbs::<_, u64>(c);

        let prod_big = a_big.clone() * b_big.clone();
        let k = prod_big.clone() / modulus_big.clone();
        println!("{}", c_big.to_str_radix(16));
        println!("{}", (a_big.clone() * b_big.clone() - k.clone() * modulus_big.clone()).to_str_radix(16));
        let mut k_bytes = k.to_bytes_le();
        assert!(k_bytes.len() <= 32);
        k_bytes.resize(32, 0);
        let k_fr = Fr::from_repr_vartime(k_bytes.try_into().unwrap()).unwrap();
        let k_limbs = split_into_limbs::<_, u64>(k_fr);

        let ab = limbs_product(&a_limbs, &b_limbs);
        let kp = limbs_product(&k_limbs, &modulus_limbs);

        let mut borrow = [0u64; 22];
        let mut tmp = ab;
        for i in 0..22 {
            if ab[i] < kp[i] {
                let borrow_bits = (kp[i] - ab[i]) / (1 << 24) + 1;
                borrow[i] = borrow_bits;
                tmp[i] -= borrow_bits;
            }
        }

        println!("modulus_limbs: {:?}", modulus_limbs);
        println!("a: {:?}", a_limbs);
        println!("b: {:?}", b_limbs);
        println!("c: {:?}", c_limbs);
        println!("k: {:?}", k_fr);
        println!("ab: {:?}", ab);
        println!("kp: {:?}", kp);
        println!("borrow: {:?}", borrow);

        let diff = ab.iter().zip(kp.iter()).map(|(a, b)| *b as i64 - *a as i64).collect::<Vec<_>>();
        println!("diff: {:?}", diff);

        let mut result = [0u64; 22];
        for i in 0..22 {
            if i > 0 {
                result[i] = ab[i] - borrow[i - 1];
            } else {
                result[i] = ab[i];
            }
            result[i] += borrow[i] * (1 << 24);

        }
        println!("result: {:?}", result);
    }
}
