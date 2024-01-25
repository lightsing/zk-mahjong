use crate::{
    sub_circuits::{
        encrypt_circuit::ElGamalEncryptCircuit,
        escalarmul_circuit::EscalarMulCircuit,
        shuffle_circuit::{dev::ShuffleTestCircuit, ShuffleCircuit},
        SubCircuit,
    },
    utils::test::{unusable_rows, TestCase},
};
use halo2_proofs::dev::MockProver;

#[test]
fn shuffle_circuit_unusable_rows() {
    assert_eq!(
        ShuffleCircuit::<144>::unusable_rows(),
        unusable_rows::<ShuffleTestCircuit::<144>>(),
    )
}

#[test]
fn test_shuffle_circuit() {
    const SIZE: usize = 144;

    let test_case = TestCase::new(SIZE);

    let elgamal = ElGamalEncryptCircuit::new(
        test_case.agg_pk,
        test_case.randomness.clone(),
        test_case.messages.clone(),
    );
    let escalarmul = EscalarMulCircuit::<{ SIZE * 2 }>::new(test_case.muls.clone());
    let shuffle = ShuffleCircuit::<SIZE>::new(
        test_case.messages.clone(),
        test_case.encrypted.clone(),
        test_case.permutation.clone(),
    );
    let test_circuit = ShuffleTestCircuit {
        elgamal,
        escalarmul,
        shuffle,
    };
    let prover = MockProver::run(17, &test_circuit, vec![]).unwrap();
    prover.assert_satisfied()
}
