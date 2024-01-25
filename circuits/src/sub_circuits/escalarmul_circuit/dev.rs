use super::{EscalarMulCircuit, EscalarMulCircuitConfig, EscalarMulCircuitConfigArgs};
use crate::sub_circuits::{
    tables::escalarmul::EscalarMulTable,
    {SubCircuit, SubCircuitConfig},
};
use halo2_proofs::{circuit::SimpleFloorPlanner, plonk::Circuit};
use halo2curves::bn256::Fr;

impl<const MAX_MULS: usize> Circuit<Fr> for EscalarMulCircuit<MAX_MULS> {
    type Config = EscalarMulCircuitConfig;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<Fr>) -> Self::Config {
        let escalarmul_table = EscalarMulTable::construct(meta);
        EscalarMulCircuitConfig::new(meta, EscalarMulCircuitConfigArgs { escalarmul_table })
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl halo2_proofs::circuit::Layouter<Fr>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        self.synthesize_sub(&config, &mut layouter)
    }
}
