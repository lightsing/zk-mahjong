use compact_str::CompactString;
use halo2_proofs::{
    circuit::Region,
    plonk::{Advice, Any, Column, ConstraintSystem, Expression, Fixed, VirtualCells},
    poly::Rotation,
};
use halo2curves::bn256::Fr;
use smallvec::SmallVec;

pub mod encrypt;
pub mod escalarmul;
pub mod fixed;

const MAX_INLINE_TABLE_COLUMNS: usize = 8;
pub type Annotations = SmallVec<CompactString, MAX_INLINE_TABLE_COLUMNS>;
pub type Columns<C> = SmallVec<Column<C>, MAX_INLINE_TABLE_COLUMNS>;
pub type TableExprs = SmallVec<Expression<Fr>, MAX_INLINE_TABLE_COLUMNS>;

/// Trait used to define lookup tables
pub trait LookupTable {
    /// Returns the list of ALL the table columns following the table order.
    fn columns(&self) -> Columns<Any>;

    /// Returns the list of ALL the table advice columns following the table
    /// order.
    fn advice_columns(&self) -> Columns<Advice> {
        self.columns()
            .iter()
            .map(|&col| col.try_into())
            .filter_map(|res| res.ok())
            .collect()
    }

    /// Returns the list of ALL the table fixed columns following the table order.
    fn fixed_columns(&self) -> Columns<Fixed> {
        self.columns()
            .iter()
            .map(|&col| col.try_into())
            .filter_map(|res| res.ok())
            .collect()
    }

    /// Returns the String annotations associated to each column of the table.
    fn annotations(&self) -> Annotations;

    /// Return the list of expressions used to define the lookup table.
    fn table_exprs(&self, meta: &mut VirtualCells<Fr>) -> TableExprs {
        self.columns()
            .iter()
            .map(|&column| meta.query_any(column, Rotation::cur()))
            .collect()
    }

    /// Annotates a lookup table by passing annotations for each of it's
    /// columns.
    fn annotate_columns(&self, cs: &mut ConstraintSystem<Fr>) {
        self.columns()
            .iter()
            .zip(self.annotations().iter())
            .for_each(|(&col, ann)| cs.annotate_lookup_any_column(col, || ann.to_string()))
    }

    /// Annotates columns of a table embedded within a circuit region.
    fn annotate_columns_in_region(&self, region: &mut Region<Fr>) {
        self.columns()
            .iter()
            .zip(self.annotations().iter())
            .for_each(|(&col, ann)| region.name_column(|| ann.to_string(), col))
    }
}
