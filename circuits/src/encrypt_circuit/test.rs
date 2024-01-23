use super::ElGamalEncryptCircuit;
use crate::{
    encrypt_circuit::dev::ElGamalEncryptTestCircuit,
    escalarmul_circuit::EscalarMulCircuit,
    utils::{unusable_rows, SubCircuit},
};
use ff::Field;
use halo2_proofs::dev::MockProver;
use halo2curves::{
    bn256::{Fq, Fr},
    group::{cofactor::CofactorCurveAffine, Curve, Group},
    grumpkin::{G1Affine, G1},
};
use rand::thread_rng;
use std::{array, iter};

#[test]
fn copy_circuit_unusable_rows() {
    assert_eq!(
        ElGamalEncryptCircuit::unusable_rows(),
        unusable_rows::<ElGamalEncryptTestCircuit>(),
    )
}

#[test]
fn test_elgamal_encrypt_circuit() {
    const SIZE: usize = 144;

    let sks: [Fq; 4] =
        array::from_fn(|_| Fq::from_bytes(&Fr::random(thread_rng()).to_bytes()).unwrap());
    let pks = sks.map(|sk| G1::generator() * sk);
    let agg_pk = pks
        .iter()
        .fold(G1::identity(), |acc, pk| acc + pk)
        .to_affine();

    let r = (0..SIZE)
        .map(|_| Fr::random(thread_rng()))
        .collect::<Vec<_>>();
    let messages = (0..SIZE)
        .map(|_| (G1Affine::identity(), G1Affine::random(thread_rng())))
        .collect::<Vec<_>>();
    // r * G and r * agg_pk
    let muls = iter::repeat(G1Affine::generator())
        .take(SIZE)
        .zip(r.iter().copied())
        .chain(iter::repeat(agg_pk).take(SIZE).zip(r.iter().copied()))
        .collect();

    let elgamal = ElGamalEncryptCircuit::new(agg_pk, r, messages);
    let escalarmul = EscalarMulCircuit::new(muls, SIZE * 2);
    let test_circuit = ElGamalEncryptTestCircuit {
        elgamal,
        escalarmul,
    };
    let prover = MockProver::run(17, &test_circuit, vec![]).unwrap();
    prover.assert_satisfied()
}
