use ff::Field;
use halo2_proofs::{
    circuit::{Layouter, Value},
    plonk::{Any, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};
use halo2curves::bn256::Fr;
use smallvec::smallvec;

use super::LookupTable;

// Lookup table for 2^i for i in 0..=254
#[derive(Clone, Copy, Debug)]
pub struct Pow2Table {
    index: Column<Fixed>,
    value: Column<Fixed>,
}

impl Pow2Table {
    // Construct the Pow2Table
    pub fn construct(meta: &mut ConstraintSystem<Fr>) -> Self {
        let index = meta.fixed_column();
        let value = meta.fixed_column();
        Pow2Table { index, value }
    }

    // Assign the value 2^i to the ith row of the table
    pub fn load(&self, layouter: &mut impl Layouter<Fr>) -> Result<(), Error> {
        layouter.assign_region(
            || "pow2 table 2^[0, 254])".to_string(),
            |mut region| {
                const F2: Fr = Fr::from_raw([2, 0, 0, 0]);
                let mut acc = Fr::ONE;
                for i in 0..255 {
                    region.assign_fixed(
                        || format!("{}", i),
                        self.index,
                        i,
                        || Value::known(Fr::from(i as u64)),
                    )?;
                    region.assign_fixed(
                        || format!("2^{}", i),
                        self.value,
                        i,
                        || Value::known(acc),
                    )?;
                    acc *= F2;
                }
                Ok(())
            },
        )
    }
}

impl LookupTable for Pow2Table {
    fn columns(&self) -> super::Columns<Any> {
        smallvec![self.index.into(), self.value.into(),]
    }

    fn annotations(&self) -> super::Annotations {
        smallvec!["pow2_table.index".into(), "pow2_table.value".into(),]
    }

    fn table_exprs(&self, meta: &mut halo2_proofs::plonk::VirtualCells<Fr>) -> super::TableExprs {
        smallvec![
            meta.query_fixed(self.index, Rotation::cur()),
            meta.query_fixed(self.value, Rotation::cur()),
        ]
    }
}
