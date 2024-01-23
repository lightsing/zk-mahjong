use ff::Field;
use halo2_proofs::{
    circuit::{Layouter, Value},
    plonk::{ConstraintSystem, Error, TableColumn},
};
use halo2curves::bn256::Fr;

// Lookup table for 2^i for i in 0..=254
#[derive(Clone, Copy, Debug)]
pub struct Pow2Table(TableColumn);

impl Pow2Table {
    // Construct the Pow2Table
    pub fn construct(meta: &mut ConstraintSystem<Fr>) -> Self {
        let inner = meta.lookup_table_column();
        meta.annotate_lookup_column(inner, || "pow2 table 2^[0, 254])".to_string());
        Pow2Table(inner)
    }

    // Assign the value 2^i to the ith row of the table
    pub fn assign(&self, layouter: &mut impl Layouter<Fr>) -> Result<(), Error> {
        layouter.assign_table(
            || "pow2 table 2^[0, 254])".to_string(),
            |mut table| {
                const F2: Fr = Fr::from_raw([2, 0, 0, 0]);
                let mut acc = Fr::ONE;
                for i in 0..255 {
                    table.assign_cell(|| format!("2^{}", i), self.0, i, || Value::known(acc))?;
                    acc *= F2;
                }
                Ok(())
            },
        )
    }
}
