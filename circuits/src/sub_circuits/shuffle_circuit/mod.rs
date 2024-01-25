use crate::{
    gadgets::utils::{and, not, pow_of_two, Expr},
    sub_circuits::{
        tables::{
            encrypt::ElGamalEncryptTable, fixed::Pow2Table, shuffle::ShuffleTable, LookupTable,
        },
        SubCircuit, SubCircuitConfig,
    },
    utils::{constraint_builder::BaseConstraintBuilder, ec::PointColumns},
};
use ff::Field;
use halo2_proofs::{
    circuit::{Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};
use halo2curves::{bn256::Fr, grumpkin::G1Affine};

#[cfg(test)]
mod dev;
#[cfg(test)]
mod test;

#[derive(Copy, Clone, Debug)]
pub struct ShuffleCircuitConfig {
    pub q_enable: Column<Fixed>,
    pub is_last: Column<Fixed>,
    pub index_pow2: Column<Fixed>,
    pub index_pow2_sum: Column<Fixed>,
    /// original index
    pub origin_index: Column<Advice>,
    /// original index pow2
    pub origin_index_pow2: Column<Advice>,
    /// sum of original index pow2
    pub origin_index_pow2_sum: Column<Advice>,
    /// The message to encrypt.
    pub cin: [PointColumns<Advice>; 2],
    /// pow2 lookup table
    pub pow2_table: Pow2Table,
    /// message encrypt table
    pub encrypt_table: ElGamalEncryptTable,
    /// shuffle output table
    pub shuffle_table: ShuffleTable,
}

pub struct ShuffleCircuitConfigArgs {
    /// pow2 lookup table
    pub pow2_table: Pow2Table,
    /// message encrypt table
    pub encrypt_table: ElGamalEncryptTable,
    /// shuffle output table
    pub shuffle_table: ShuffleTable,
}

impl SubCircuitConfig for ShuffleCircuitConfig {
    type ConfigArgs = ShuffleCircuitConfigArgs;

    fn new(
        meta: &mut ConstraintSystem<Fr>,
        ShuffleCircuitConfigArgs {
            pow2_table,
            encrypt_table,
            shuffle_table,
        }: Self::ConfigArgs,
    ) -> Self {
        let q_enable = shuffle_table.q_enable;
        let is_last = meta.fixed_column();
        let index_pow2 = meta.fixed_column();
        let index_pow2_sum = meta.fixed_column();
        let origin_index = meta.advice_column();
        let origin_index_pow2 = meta.advice_column();
        let origin_index_pow2_sum = meta.advice_column();
        let cin = [
            PointColumns::<Advice>::construct(meta),
            PointColumns::<Advice>::construct(meta),
        ];

        meta.create_gate("verify sum", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            cb.require_equal(
                "origin_index_pow2_sum::next = origin_index_pow2::next + origin_index_pow2_sum::cur",
                meta.query_advice(origin_index_pow2_sum, Rotation::next()),
                meta.query_advice(origin_index_pow2, Rotation::next())
                    + meta.query_advice(origin_index_pow2_sum, Rotation::cur()),
            );

            cb.gate(and::expr([
                meta.query_fixed(q_enable, Rotation::cur()),
                not::expr(meta.query_fixed(is_last, Rotation::cur())),
            ]))
        });

        meta.create_gate("verify last sum", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            let index_pow2_sum = meta.query_fixed(index_pow2_sum, Rotation::cur());
            let origin_index_pow2_sum = meta.query_advice(origin_index_pow2_sum, Rotation::cur());
            cb.require_equal("sum must equal", index_pow2_sum, origin_index_pow2_sum);

            cb.gate(meta.query_fixed(is_last, Rotation::cur()))
        });

        meta.lookup_any("pow2 holds", |meta| {
            let q_enable = meta.query_fixed(q_enable, Rotation::cur());
            [
                meta.query_advice(origin_index, Rotation::cur()),
                meta.query_advice(origin_index_pow2, Rotation::cur()),
            ]
            .into_iter()
            .zip(pow2_table.table_exprs(meta))
            .map(|(arg, table)| (q_enable.clone() * arg, table))
            .collect()
        });

        meta.lookup_any("origin message exists", |meta| {
            let q_enable = meta.query_fixed(q_enable, Rotation::cur());
            [
                1.expr(),
                meta.query_advice(encrypt_table.agg_pk.x, Rotation::cur()),
                meta.query_advice(encrypt_table.agg_pk.y, Rotation::cur()),
                meta.query_advice(origin_index, Rotation::cur()),
                meta.query_advice(cin[0].x, Rotation::cur()),
                meta.query_advice(cin[0].y, Rotation::cur()),
                meta.query_advice(cin[1].x, Rotation::cur()),
                meta.query_advice(cin[1].y, Rotation::cur()),
                meta.query_advice(shuffle_table.shuffled[0].x, Rotation::cur()),
                meta.query_advice(shuffle_table.shuffled[0].y, Rotation::cur()),
                meta.query_advice(shuffle_table.shuffled[1].x, Rotation::cur()),
                meta.query_advice(shuffle_table.shuffled[1].y, Rotation::cur()),
            ]
            .into_iter()
            .zip(encrypt_table.table_exprs(meta))
            .map(|(arg, table)| (q_enable.clone() * arg, table))
            .collect()
        });

        ShuffleCircuitConfig {
            q_enable,
            is_last,
            index_pow2,
            index_pow2_sum,
            origin_index,
            origin_index_pow2,
            origin_index_pow2_sum,
            cin,
            pow2_table,
            encrypt_table,
            shuffle_table,
        }
    }
}

impl ShuffleCircuitConfig {
    pub fn assign_messages(
        &self,
        layouter: &mut impl Layouter<Fr>,
        n: usize,
        messages: &[(G1Affine, G1Affine)],
        encrypted: &[(G1Affine, G1Affine)],
        permutation: &[usize],
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "shuffle circuit",
            |mut region| {
                region.name_column(|| "q_enable", self.q_enable);
                region.name_column(|| "index", self.shuffle_table.index);
                region.name_column(|| "index_pow2", self.index_pow2);
                region.name_column(|| "index_pow2_sum", self.index_pow2_sum);
                region.name_column(|| "origin_index", self.origin_index);
                region.name_column(|| "origin_index_pow2", self.origin_index_pow2);
                region.name_column(|| "origin_index_pow2_sum", self.origin_index_pow2_sum);
                self.cin[0].name_columns(&mut region, "cin[0]");
                self.cin[1].name_columns(&mut region, "cin[1]");
                self.shuffle_table.shuffled[0].name_columns(&mut region, "cout[0]");
                self.shuffle_table.shuffled[1].name_columns(&mut region, "cout[1]");

                for index in 0..n {
                    region.assign_fixed(
                        || "q_enable",
                        self.q_enable,
                        index,
                        || Value::known(Fr::ONE),
                    )?;
                    region.assign_fixed(
                        || "is_last",
                        self.is_last,
                        index,
                        || Value::known(Fr::from(index == n - 1)),
                    )?;
                    region.assign_fixed(
                        || "index",
                        self.shuffle_table.index,
                        index,
                        || Value::known(Fr::from(index as u64)),
                    )?;
                    region.assign_fixed(
                        || "index_pow2",
                        self.index_pow2,
                        index,
                        || Value::known(pow_of_two::<Fr>(index)),
                    )?;
                    region.assign_fixed(
                        || "index_pow2_sum",
                        self.index_pow2_sum,
                        index,
                        || Value::known(pow_of_two::<Fr>(index + 1) - Fr::ONE),
                    )?;
                }

                let mut origin_index_pow2_sum = Fr::ZERO;
                for (index, origin_index) in permutation.iter().copied().enumerate() {
                    region.assign_advice(
                        || "origin_index",
                        self.origin_index,
                        index,
                        || Value::known(Fr::from(origin_index as u64)),
                    )?;
                    let origin_index_pow2 = pow_of_two::<Fr>(origin_index);
                    region.assign_advice(
                        || "origin_index_pow2",
                        self.origin_index_pow2,
                        index,
                        || Value::known(origin_index_pow2),
                    )?;
                    origin_index_pow2_sum += origin_index_pow2;
                    region.assign_advice(
                        || "origin_index_pow2_sum",
                        self.origin_index_pow2_sum,
                        index,
                        || Value::known(origin_index_pow2_sum),
                    )?;
                    self.cin[0].assign("cin0", &mut region, index, &messages[origin_index].0)?;
                    self.cin[1].assign("cin1", &mut region, index, &messages[origin_index].1)?;
                    self.shuffle_table.shuffled[0].assign(
                        "cout0",
                        &mut region,
                        index,
                        &encrypted[origin_index].0,
                    )?;
                    self.shuffle_table.shuffled[1].assign(
                        "cout1",
                        &mut region,
                        index,
                        &encrypted[origin_index].1,
                    )?;
                }
                Ok(())
            },
        )
    }
}

#[derive(Default, Clone, Debug)]
pub struct ShuffleCircuit<const N: usize> {
    pub messages: Vec<(G1Affine, G1Affine)>,
    pub encrypted: Vec<(G1Affine, G1Affine)>,
    pub permutation: Vec<usize>,
}

impl<const N: usize> ShuffleCircuit<N> {
    pub fn new(
        messages: Vec<(G1Affine, G1Affine)>,
        encrypted: Vec<(G1Affine, G1Affine)>,
        permutation: Vec<usize>,
    ) -> Self {
        Self {
            messages,
            encrypted,
            permutation,
        }
    }
}

impl<const N: usize> SubCircuit for ShuffleCircuit<N> {
    type Config = ShuffleCircuitConfig;

    fn synthesize_sub(
        &self,
        config: &Self::Config,
        layouter: &mut impl Layouter<Fr>,
    ) -> Result<(), Error> {
        config.assign_messages(
            layouter,
            N,
            &self.messages,
            &self.encrypted,
            &self.permutation,
        )
    }
}
