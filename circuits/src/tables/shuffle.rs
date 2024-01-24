use super::LookupTable;
use crate::utils::ec::PointColumns;
use halo2_proofs::plonk::{Advice, Any, Column, ConstraintSystem, Fixed};
use halo2curves::bn256::Fr;
use smallvec::smallvec;

/// Lookup table within the ElGamalEncrypt circuit.
#[derive(Clone, Copy, Debug)]
pub struct ShuffleTable {
    /// Whether the row is enabled.
    pub q_enable: Column<Fixed>,
    /// The index of the row.
    pub index: Column<Fixed>,
    /// The shuffled message.
    pub shuffled: [PointColumns<Advice>; 2],
}

impl ShuffleTable {
    /// Construct the Shuffle table.
    pub fn construct(meta: &mut ConstraintSystem<Fr>) -> Self {
        let q_enable = meta.fixed_column();
        let index = meta.fixed_column();
        let shuffled = [
            PointColumns::<Advice>::construct(meta),
            PointColumns::<Advice>::construct(meta),
        ];
        ShuffleTable {
            q_enable,
            index,
            shuffled,
        }
    }
}

impl LookupTable for ShuffleTable {
    fn columns(&self) -> super::Columns<Any> {
        smallvec![
            self.q_enable.into(),
            self.index.into(),
            self.shuffled[0].x.into(),
            self.shuffled[0].y.into(),
            self.shuffled[1].x.into(),
            self.shuffled[1].y.into(),
        ]
    }

    fn annotations(&self) -> super::Annotations {
        smallvec![
            "q_enable".into(),
            "index".into(),
            "shuffled[0].x".into(),
            "shuffled[0].y".into(),
            "shuffled[1].x".into(),
            "shuffled[1].y".into(),
        ]
    }
}
