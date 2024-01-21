use ff::PrimeField;
use halo2_proofs::{plonk::{Column, Fixed, Advice, ConstraintSystem}, circuit::Value};
use crate::escalarmul_circuit::PointColumns;

/// Lookup table within the Babyjubjub Scalar Mul circuit.
#[derive(Clone, Copy, Debug)]
pub struct EscalarMulTable {
    /// Whether the row is enabled.
    pub q_enable: Column<Fixed>,
    /// Whether this row is the last row in the Babyjubjub Scalar Mul's trace.
    pub is_last: Column<Advice>,
    /// accumulator of the scalar
    pub scalar_acc: Column<Advice>,
    /// base in twisted Edwards point form
    pub base: PointColumns,
    /// result of the scalar multiplication in twisted Edwards point form
    pub result: PointColumns,
}

pub struct EscalarMulAssignRow<F: PrimeField> {
    pub scalar_bit: F,
    pub scalar_acc: F,

}

impl EscalarMulTable {
    /// Construct the Babyjubjub Scalar Mul table.
    pub fn construct<F: PrimeField>(meta: &mut ConstraintSystem<F>) -> Self {
        let q_enable = meta.fixed_column();
        let is_last = meta.advice_column();
        let scalar_acc = meta.advice_column();
        let base = PointColumns::construct(meta);
        let result = PointColumns::construct(meta);
        EscalarMulTable {
            q_enable,
            is_last,
            scalar_acc,
            base,
            result,
        }
    }

    pub fn assign<F: PrimeField>(
        base: (F, F),
        scalar: F,
    ) -> Vec<EscalarMulAssignRow<F>> {
        todo!()
    }
}