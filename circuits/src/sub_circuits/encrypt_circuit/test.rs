use super::ElGamalEncryptCircuit;
use crate::{
    sub_circuits::{
        encrypt_circuit::dev::ElGamalEncryptTestCircuit, escalarmul_circuit::EscalarMulCircuit,
        SubCircuit,
    },
    utils::test::{unusable_rows, TestCase},
};
use halo2_proofs::dev::MockProver;

#[test]
fn copy_circuit_unusable_rows() {
    assert_eq!(
        ElGamalEncryptCircuit::<144>::unusable_rows(),
        unusable_rows::<ElGamalEncryptTestCircuit::<144>>(),
    )
}

#[test]
fn test_elgamal_encrypt_circuit() {
    const SIZE: usize = 144;

    let test_case = TestCase::new(SIZE);

    let elgamal = ElGamalEncryptCircuit::<SIZE>::new(
        test_case.agg_pk,
        test_case.randomness.clone(),
        test_case.messages.clone(),
    );
    let escalarmul = EscalarMulCircuit::<{ SIZE * 2 }>::new(test_case.muls.clone());
    let test_circuit = ElGamalEncryptTestCircuit {
        elgamal,
        escalarmul,
    };
    let prover = MockProver::run(17, &test_circuit, vec![]).unwrap();
    prover.assert_satisfied()
}
