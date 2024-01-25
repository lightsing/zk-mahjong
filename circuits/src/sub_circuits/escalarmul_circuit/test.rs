use super::EscalarMulCircuit;
use crate::sub_circuits::SubCircuit;
use crate::utils::test::unusable_rows;
use halo2_proofs::dev::MockProver;
use halo2curves::{bn256::Fr, grumpkin::G1Affine};

#[test]
fn copy_circuit_unusable_rows() {
    assert_eq!(
        EscalarMulCircuit::<288>::unusable_rows(),
        unusable_rows::<EscalarMulCircuit::<288>>(),
    )
}

#[test]
fn test_escalarmul_circuit() {
    const MAX_MULS: usize = 288;
    let muls = (0..MAX_MULS as u64)
        .map(|i| (G1Affine::generator(), Fr::from(i + 1)))
        .collect::<Vec<_>>();
    let circuit = EscalarMulCircuit::<MAX_MULS>::new(muls);
    let prover = MockProver::run(17, &circuit, vec![]).unwrap();
    prover.assert_satisfied()
}
