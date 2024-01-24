use halo2_proofs::{
    circuit::Layouter,
    plonk::{ConstraintSystem, Error},
};
use halo2curves::bn256::Fr;

pub mod constraint_builder;
pub mod ec;
#[cfg(test)]
pub mod test;

/// SubCircuit is a circuit that performs the verification of a specific part of
/// the full Ethereum block verification.  The SubCircuit's interact with each
/// other via lookup tables and/or shared public inputs.  This type must contain
/// all the inputs required to synthesize this circuit (and the contained
/// table(s) if any).
pub trait SubCircuit {
    /// Configuration of the SubCircuit.
    type Config: SubCircuitConfig;

    /// Returns number of unusable rows of the SubCircuit, which should be
    /// `meta.blinding_factors() + 1`.
    fn unusable_rows() -> usize {
        256
    }

    /// Returns the instance columns required for this circuit.
    fn instance(&self) -> Vec<Vec<Fr>> {
        vec![]
    }

    /// Assign only the columns used by this sub-circuit.  This includes the
    /// columns that belong to the exposed lookup table contained within, if
    /// any; and excludes external tables that this sub-circuit does lookups
    /// to.
    fn synthesize_sub(
        &self,
        config: &Self::Config,
        layouter: &mut impl Layouter<Fr>,
    ) -> Result<(), Error>;
}

/// SubCircuit configuration
pub trait SubCircuitConfig {
    /// Config constructor arguments
    type ConfigArgs;

    /// Type constructor
    fn new(meta: &mut ConstraintSystem<Fr>, args: Self::ConfigArgs) -> Self;
}

/// Returns number of unusable rows of the Circuit.
/// The minimum unusable rows of a circuit is currently 6, where
/// - 3 comes from minimum number of distinct queries to permutation argument witness column
/// - 1 comes from queries at x_3 during multiopen
/// - 1 comes as slight defense against off-by-one errors
/// - 1 comes from reservation for last row for grand-product boundray check, hence not copy-able or
///   lookup-able. Note this 1 is not considered in [`ConstraintSystem::blinding_factors`], so below
///   we need to add an extra 1.
///
/// For circuit with column queried at more than 3 distinct rotation, we can
/// calculate the unusable rows as (x - 3) + 6 where x is the number of distinct
/// rotation.
#[cfg(test)]
pub(crate) fn unusable_rows<C: halo2_proofs::plonk::Circuit<Fr>>() -> usize {
    let mut cs = ConstraintSystem::default();
    C::configure(&mut cs);

    cs.blinding_factors() + 1
}
