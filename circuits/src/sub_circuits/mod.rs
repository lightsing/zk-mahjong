use halo2_proofs::{
    circuit::Layouter,
    plonk::{ConstraintSystem, Error},
};
use halo2curves::bn256::Fr;

//pub mod encrypt_circuit;
pub mod escalarmul_circuit;
//pub mod shuffle_circuit;
pub mod tables;

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
