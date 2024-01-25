use super::LookupTable;
use crate::utils::ec::PointColumns;
use compact_str::CompactString;
use ff::{Field, PrimeField};
use halo2_proofs::plonk::{Advice, Any, Column, ConstraintSystem, Fixed};
use halo2curves::{
    bn256::{Fq, Fr},
    group::{Curve, Group},
    grumpkin::{G1Affine, G1},
};
use smallvec::smallvec;

/// Lookup table within the EC Scalar Mul circuit.
#[derive(Clone, Copy, Debug)]
pub struct EscalarMulTable {
    /// Whether the row is enabled.
    pub q_enable: Column<Fixed>,
    /// Whether this row is the last row in the EC Scalar Mul's trace.
    pub is_last: Column<Advice>,
    /// accumulator of the scalar
    pub scalar_acc: Column<Advice>,
    /// base in twisted Edwards point form
    pub base: PointColumns<Advice>,
    /// result of the scalar multiplication in twisted Edwards point form
    pub result: PointColumns<Advice>,
}

#[derive(Debug)]
pub struct EscalarMulAssignRow {
    pub scalar_bit: Fr,
    pub scalar_acc: Fr,
    pub result: G1,
    pub result2: G1,
}

impl EscalarMulTable {
    /// Construct the Babyjubjub Scalar Mul table.
    pub fn construct(meta: &mut ConstraintSystem<Fr>) -> Self {
        let q_enable = meta.fixed_column();
        let is_last = meta.advice_column();
        let scalar_acc = meta.advice_column();
        let base = PointColumns::<Advice>::construct(meta);
        let result = PointColumns::<Advice>::construct(meta);
        EscalarMulTable {
            q_enable,
            is_last,
            scalar_acc,
            base,
            result,
        }
    }

    pub fn assign(base: G1Affine, scalar: Fr) -> Vec<EscalarMulAssignRow> {
        let mut assignment = Vec::with_capacity(Fr::NUM_BITS as usize + 1);
        let mut scalar_acc = Fr::ZERO;
        let mut result = G1::identity();
        #[cfg(debug_assertions)]
        let expected = base * Fq::from_repr(scalar.to_repr()).unwrap();

        // MSB iter
        for bit in scalar
            .to_repr()
            .iter()
            .rev()
            .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1u8))
        {
            let scalar_bit = Fr::from(bit as u64);
            scalar_acc = scalar_acc * Fr::from(2) + scalar_bit;
            let result2 = result.double();
            assignment.push(EscalarMulAssignRow {
                scalar_bit,
                scalar_acc,
                result,
                result2,
            });
            if bit == 1 {
                result = result2 + base;
            } else {
                result = result2;
            }
        }
        assignment.push(EscalarMulAssignRow {
            scalar_bit: Fr::ZERO,
            scalar_acc,
            result,
            result2: result.double(),
        });

        #[cfg(debug_assertions)]
        assert_eq!(expected.to_affine(), result.to_affine(), "{assignment:#?}");

        assignment
    }
}

impl LookupTable for EscalarMulTable {
    fn columns(&self) -> super::Columns<Any> {
        smallvec![
            self.q_enable.into(),
            self.is_last.into(),
            self.scalar_acc.into(),
            self.base.x.into(),
            self.base.y.into(),
            self.result.x.into(),
            self.result.y.into(),
        ]
    }

    fn annotations(&self) -> super::Annotations {
        smallvec![
            CompactString::new("q_enable"),
            CompactString::new("is_last"),
            CompactString::new("scalar_acc"),
            CompactString::new("base.x"),
            CompactString::new("base.y"),
            CompactString::new("result.x"),
            CompactString::new("result.y"),
        ]
    }
}
