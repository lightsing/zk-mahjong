use crate::sub_circuits::{
    encrypt_circuit::{
        ElGamalEncryptCircuit, ElGamalEncryptCircuitConfig, ElGamalEncryptCircuitConfigArgs,
    },
    escalarmul_circuit::{EscalarMulCircuit, EscalarMulCircuitConfig, EscalarMulCircuitConfigArgs},
    tables::{
        encrypt::ElGamalEncryptTable, escalarmul::EscalarMulTable, fixed::Pow2Table,
        shuffle::ShuffleTable,
    },
    SubCircuit, SubCircuitConfig,
};

use super::{ShuffleCircuit, ShuffleCircuitConfig, ShuffleCircuitConfigArgs};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Circuit, ConstraintSystem, Error},
};
use halo2curves::bn256::Fr;

#[derive(Default)]
pub struct ShuffleTestCircuit<const N: usize>
where
    [(); N * 2]:,
{
    pub elgamal: ElGamalEncryptCircuit<N>,
    pub escalarmul: EscalarMulCircuit<{ N * 2 }>,
    pub shuffle: ShuffleCircuit<N>,
}

#[derive(Clone)]
pub struct ShuffleTestCircuitConfig {
    pub pow2_table: Pow2Table,
    pub elgamal: ElGamalEncryptCircuitConfig,
    pub escalarmul: EscalarMulCircuitConfig,
    pub shuffle: ShuffleCircuitConfig,
}

impl<const N: usize> Circuit<Fr> for ShuffleTestCircuit<N>
where
    [(); N * 2]:,
{
    type Config = ShuffleTestCircuitConfig;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        let escalarmul_table = EscalarMulTable::construct(meta);
        let encrypt_table = ElGamalEncryptTable::construct(meta);
        let shuffle_table = ShuffleTable::construct(meta);
        let pow2_table = Pow2Table::construct(meta);

        ShuffleTestCircuitConfig {
            pow2_table,
            elgamal: ElGamalEncryptCircuitConfig::new(
                meta,
                ElGamalEncryptCircuitConfigArgs {
                    escalarmul_table,
                    encrypt_table,
                },
            ),
            escalarmul: EscalarMulCircuitConfig::new(
                meta,
                EscalarMulCircuitConfigArgs { escalarmul_table },
            ),
            shuffle: ShuffleCircuitConfig::new(
                meta,
                ShuffleCircuitConfigArgs {
                    pow2_table,
                    encrypt_table,
                    shuffle_table,
                },
            ),
        }
    }

    fn synthesize(
        &self,
        Self::Config {
            pow2_table,
            elgamal,
            escalarmul,
            shuffle,
        }: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        pow2_table.load(&mut layouter)?;
        self.escalarmul.synthesize_sub(&escalarmul, &mut layouter)?;
        self.elgamal.synthesize_sub(&elgamal, &mut layouter)?;
        self.shuffle.synthesize_sub(&shuffle, &mut layouter)?;
        Ok(())
    }
}
