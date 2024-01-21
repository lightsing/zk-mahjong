use ff::PrimeField;
use halo2_proofs::{plonk::{Column, Fixed, Advice, ConstraintSystem}, halo2curves::CurveAffine};
use crate::tables::escalarmul::EscalarMulTable;


#[derive(Copy, Clone, Debug)]
pub struct ProjectivePointColumns {
    pub x: Column<Advice>,
    pub y: Column<Advice>,
    pub z: Column<Advice>,
}

impl ProjectivePointColumns {
    pub fn construct<F: PrimeField>(meta: &mut ConstraintSystem<F>) -> Self {
        let x = meta.advice_column();
        let y = meta.advice_column();
        let z = meta.advice_column();
        ProjectivePointColumns { x, y, z }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PointColumns {
    pub x: Column<Advice>,
    pub y: Column<Advice>,
}

impl PointColumns {
    pub fn construct<F: PrimeField>(meta: &mut ConstraintSystem<F>) -> Self {
        let x = meta.advice_column();
        let y = meta.advice_column();
        PointColumns { x, y }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BabyjubjubCircuitConfig {
    /// Whether the row is enabled.
    pub q_enable: Column<Fixed>,
    /// bit decomposition of the scalar
    pub scalar_bit: Column<Advice>,
    /// result of the scalar multiplication
    pub result: ProjectivePointColumns,
    /// exponent accumulator
    pub exp_acc: Column<Advice>,
    /// auxiliary variable for the exponentiation = result + exp_acc
    pub exp_aux: Column<Advice>,
    /// lookup table
    pub escalarmul_table: EscalarMulTable,
}