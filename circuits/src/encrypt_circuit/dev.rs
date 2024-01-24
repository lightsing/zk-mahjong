use super::{ElGamalEncryptCircuit, ElGamalEncryptCircuitConfig, ElGamalEncryptCircuitConfigArgs};
use crate::{
    escalarmul_circuit::{EscalarMulCircuit, EscalarMulCircuitConfig, EscalarMulCircuitConfigArgs},
    tables::{encrypt::ElGamalEncryptTable, escalarmul::EscalarMulTable},
    utils::{SubCircuit, SubCircuitConfig},
};
use halo2_proofs::{circuit::SimpleFloorPlanner, plonk::Circuit};
use halo2curves::bn256::Fr;

#[derive(Default)]
pub struct ElGamalEncryptTestCircuit {
    pub elgamal: ElGamalEncryptCircuit,
    pub escalarmul: EscalarMulCircuit,
}

impl Circuit<Fr> for ElGamalEncryptTestCircuit {
    type Config = (ElGamalEncryptCircuitConfig, EscalarMulCircuitConfig);

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<Fr>) -> Self::Config {
        let escalarmul_table = EscalarMulTable::construct(meta);
        let encrypt_table = ElGamalEncryptTable::construct(meta);

        (
            ElGamalEncryptCircuitConfig::new(
                meta,
                ElGamalEncryptCircuitConfigArgs {
                    encrypt_table,
                    escalarmul_table,
                },
            ),
            EscalarMulCircuitConfig::new(meta, EscalarMulCircuitConfigArgs { escalarmul_table }),
        )
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl halo2_proofs::circuit::Layouter<Fr>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        self.escalarmul.synthesize_sub(&config.1, &mut layouter)?;
        self.elgamal.synthesize_sub(&config.0, &mut layouter)?;
        Ok(())
    }
}
