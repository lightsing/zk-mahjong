use crate::{
    shuffle_encrypt_circuit::ShuffleEncryptCircuit,
    sub_circuits::{
        encrypt_circuit::ElGamalEncryptCircuit, escalarmul_circuit::EscalarMulCircuit,
        shuffle_circuit::ShuffleCircuit,
    },
    utils::test::TestCase,
};
use halo2_proofs::dev::MockProver;
use halo2curves::{
    group::{Curve, Group},
    grumpkin::G1,
};

#[test]
fn test_shuffle_encrypt() {
    const SIZE: usize = 144;

    let test_case = TestCase::new(SIZE);

    let encrypt_circuit = ElGamalEncryptCircuit::new(
        test_case.agg_pk,
        test_case.randomness.clone(),
        test_case.messages.clone(),
    );
    let escalarmul_circuit = EscalarMulCircuit::<{ SIZE * 2 }>::new(test_case.muls.clone());
    let shuffle_circuit = ShuffleCircuit::<SIZE>::new(
        test_case.messages.clone(),
        test_case.encrypted.clone(),
        test_case.permutation.clone(),
    );
    let shuffle_encrypt_circuit = ShuffleEncryptCircuit::<SIZE> {
        escalarmul_circuit,
        encrypt_circuit,
        shuffle_circuit,
    };

    let input_sum = test_case
        .messages
        .iter()
        .fold(G1::identity(), |acc, m| acc + m.0 + m.1)
        .to_affine();
    let output_sum = test_case
        .encrypted
        .iter()
        .fold(G1::identity(), |acc, m| acc + m.0 + m.1)
        .to_affine();

    let prover = MockProver::run(
        17,
        &shuffle_encrypt_circuit,
        vec![
            vec![test_case.agg_pk.x, test_case.agg_pk.y],
            vec![input_sum.x, input_sum.y, output_sum.x, output_sum.y],
        ],
    )
    .unwrap();
    prover.assert_satisfied()
}
