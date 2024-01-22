use crate::utils::{unusable_rows, SubCircuit};

use super::EscalarMulCircuit;
use halo2_proofs::dev::MockProver;
use halo2curves::{bn256::Fr, grumpkin::G1Affine};

#[test]
fn copy_circuit_unusable_rows() {
    assert_eq!(
        EscalarMulCircuit::unusable_rows(),
        unusable_rows::<EscalarMulCircuit>(),
    )
}

#[test]
fn test_escalarmul_circuit() {
    let muls = (0..288)
        .map(|i| (G1Affine::generator(), Fr::from(i + 1)))
        .collect::<Vec<_>>();
    let circuit = EscalarMulCircuit::new(muls, 288);
    let prover = MockProver::run(17, &circuit, vec![]).unwrap();
    prover.assert_satisfied()
}
