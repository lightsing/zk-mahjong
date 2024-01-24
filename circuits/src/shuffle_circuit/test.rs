use crate::{
    encrypt_circuit::ElGamalEncryptCircuit,
    escalarmul_circuit::EscalarMulCircuit,
    shuffle_circuit::{dev::ShuffleTestCircuit, ShuffleCircuit},
    utils::{test::TestCase, unusable_rows, SubCircuit},
};
use halo2_proofs::dev::MockProver;

#[test]
fn shuffle_circuit_unusable_rows() {
    assert_eq!(
        ShuffleCircuit::unusable_rows(),
        unusable_rows::<ShuffleTestCircuit>(),
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
    let escalarmul = EscalarMulCircuit::new(test_case.muls.clone(), SIZE * 2);
    let shuffle = ShuffleCircuit::new(
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
