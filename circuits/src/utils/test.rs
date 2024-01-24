use ff::Field;
use halo2curves::{
    bn256::{Fq, Fr},
    group::{prime::PrimeCurveAffine, Curve, Group},
    grumpkin::{G1Affine, G1},
};
use rand::{seq::IteratorRandom, thread_rng};
use std::{array, iter};
use zk_mahjong_core::elgamal::MaskedMessage;

pub struct TestCase {
    pub sks: [Fq; 4],
    pub pks: [G1; 4],
    pub agg_pk: G1Affine,
    pub messages: Vec<(G1Affine, G1Affine)>,
    pub randomness: Vec<Fr>,
    pub muls: Vec<(G1Affine, Fr)>,
    pub permutation: Vec<usize>,
    pub encrypted: Vec<(G1Affine, G1Affine)>,
    pub shuffled: Vec<(G1Affine, G1Affine)>,
}

impl TestCase {
    pub fn new(size: usize) -> TestCase {
        let sks: [Fq; 4] =
            array::from_fn(|_| Fq::from_bytes(&Fr::random(thread_rng()).to_bytes()).unwrap());
        let pks = sks.map(|sk| G1::generator() * sk);
        let agg_pk = pks
            .iter()
            .fold(G1::identity(), |acc, pk| acc + pk)
            .to_affine();

        let messages = (0..size)
            .map(|_| (G1Affine::identity(), G1Affine::random(thread_rng())))
            .collect::<Vec<_>>();

        let randomness = (0..size)
            .map(|_| Fr::random(thread_rng()))
            .collect::<Vec<_>>();

        let encrypted = messages
            .iter()
            .map(|m| MaskedMessage {
                c0: m.0.to_curve(),
                c1: m.1.to_curve(),
            })
            .zip(randomness.iter())
            .map(|(m, r)| m.remask(&agg_pk.to_curve(), &Fq::from_bytes(&r.to_bytes()).unwrap()))
            .map(|m| (m.c0.to_affine(), m.c1.to_affine()))
            .collect::<Vec<_>>();

        // Fisher–Yates shuffle
        let mut permutation = (0..size).collect::<Vec<usize>>();
        for i in 0..size {
            let j = (i..size).choose(&mut thread_rng()).unwrap();
            permutation.swap(i, j);
        }
        let shuffled = permutation.iter().map(|&i| messages[i]).collect::<Vec<_>>();

        // r * G and r * agg_pk
        let muls = iter::repeat(G1Affine::generator())
            .take(size)
            .zip(randomness.iter().copied())
            .chain(
                iter::repeat(agg_pk)
                    .take(size)
                    .zip(randomness.iter().copied()),
            )
            .collect();

        TestCase {
            sks,
            pks,
            agg_pk,
            messages,
            randomness,
            muls,
            encrypted,
            permutation,
            shuffled,
        }
    }
}
