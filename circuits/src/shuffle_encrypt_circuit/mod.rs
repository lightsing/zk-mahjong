use crate::{
    gadgets::utils::{and, not, Expr},
    sub_circuits::{
        encrypt_circuit::{
            ElGamalEncryptCircuit, ElGamalEncryptCircuitConfig, ElGamalEncryptCircuitConfigArgs,
        },
        escalarmul_circuit::{
            EscalarMulCircuit, EscalarMulCircuitConfig, EscalarMulCircuitConfigArgs,
        },
        shuffle_circuit::{ShuffleCircuit, ShuffleCircuitConfig, ShuffleCircuitConfigArgs},
        tables::{
            encrypt::ElGamalEncryptTable, escalarmul::EscalarMulTable, fixed::Pow2Table,
            shuffle::ShuffleTable,
        },
        SubCircuit, SubCircuitConfig,
    },
    utils::{
        constraint_builder::BaseConstraintBuilder,
        ec::{PointColumns, ProjectivePointColumns},
    },
};
use ff::Field;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::Rotation,
};
use halo2curves::{
    bn256::Fr,
    group::{Curve, Group},
    grumpkin::{G1Affine, G1},
};

#[cfg(test)]
mod test;

#[derive(Clone, Debug)]
pub struct ShuffleEncryptCircuitConfig {
    pub q_enable: Column<Fixed>,
    pub is_first: Column<Fixed>,
    pub is_last: Column<Fixed>,
    pub inputs: Column<Instance>,
    // cin_sum = encrypt_table.cin[0] + encrypt_table.cin[1]
    pub cin_sum: ProjectivePointColumns,
    // cin_sum_acc::next = cin_sum_acc::cur + cin_sum::cur
    pub cin_sum_acc: ProjectivePointColumns,
    pub cin_sum_acc_affine: PointColumns<Advice>,
    // cout_sum = shuffle_table.shuffled[0] + shuffle_table.shuffled[1]
    pub cout_sum: ProjectivePointColumns,
    // cout_sum_acc::next = cout_sum_acc::cur + cout_sum::cur
    pub cout_sum_acc: ProjectivePointColumns,
    pub cout_sum_acc_affine: PointColumns<Advice>,
    pub pow2_table: Pow2Table,
    pub escalarmul_config: EscalarMulCircuitConfig,
    pub encrypt_config: ElGamalEncryptCircuitConfig,
    pub shuffle_config: ShuffleCircuitConfig,
}

pub struct ShuffleEncryptCircuitConfigArgs {
    pub pow2_table: Pow2Table,
    pub escalarmul_config: EscalarMulCircuitConfig,
    pub encrypt_config: ElGamalEncryptCircuitConfig,
    pub shuffle_config: ShuffleCircuitConfig,
}

impl SubCircuitConfig for ShuffleEncryptCircuitConfig {
    type ConfigArgs = ShuffleEncryptCircuitConfigArgs;

    fn new(
        meta: &mut ConstraintSystem<Fr>,
        ShuffleEncryptCircuitConfigArgs {
            pow2_table,
            escalarmul_config,
            encrypt_config,
            shuffle_config,
        }: Self::ConfigArgs,
    ) -> Self {
        let q_enable = meta.fixed_column();
        let is_first = meta.fixed_column();
        let is_last = meta.fixed_column();
        let inputs = meta.instance_column();
        let cin_sum = ProjectivePointColumns::construct(meta);
        let cin_sum_acc = ProjectivePointColumns::construct(meta);
        let cin_sum_acc_affine = PointColumns::<Advice>::construct(meta);
        let cout_sum = ProjectivePointColumns::construct(meta);
        let cout_sum_acc = ProjectivePointColumns::construct(meta);
        let cout_sum_acc_affine = PointColumns::<Advice>::construct(meta);

        meta.enable_equality(inputs);
        cin_sum_acc_affine.enable_equality(meta);
        cout_sum_acc_affine.enable_equality(meta);

        meta.create_gate("constraint sum first row", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            for (l, r) in cin_sum
                .columns()
                .into_iter()
                .zip(cin_sum_acc.columns())
                .chain(cout_sum.columns().into_iter().zip(cout_sum_acc.columns()))
            {
                cb.require_equal(
                    "intial sum",
                    meta.query_advice(l, Rotation::cur()),
                    meta.query_advice(r, Rotation::cur()),
                );
            }

            cb.gate(meta.query_fixed(is_first, Rotation::cur()))
        });

        meta.create_gate("constraint sum each row but not last", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            for ([in_x, in_y, in_z], [acc_x, acc_y, acc_z]) in [
                (cin_sum.columns(), cin_sum_acc.columns()),
                (cout_sum.columns(), cout_sum_acc.columns()),
            ] {
                ProjectivePointColumns::constraint_add(
                    meta,
                    &mut cb,
                    |meta| meta.query_advice(acc_x, Rotation::cur()),
                    |meta| meta.query_advice(acc_y, Rotation::cur()),
                    |meta| meta.query_advice(acc_z, Rotation::cur()),
                    |meta| meta.query_advice(in_x, Rotation::next()),
                    |meta| meta.query_advice(in_y, Rotation::next()),
                    |meta| meta.query_advice(in_z, Rotation::next()),
                    |meta| meta.query_advice(acc_x, Rotation::next()),
                    |meta| meta.query_advice(acc_y, Rotation::next()),
                    |meta| meta.query_advice(acc_z, Rotation::next()),
                )
            }

            cb.gate(and::expr([
                meta.query_fixed(q_enable, Rotation::cur()),
                not::expr(meta.query_fixed(is_last, Rotation::cur())),
            ]))
        });

        meta.create_gate("constraint each row", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            for cols in [cin_sum, cin_sum_acc, cout_sum, cout_sum_acc] {
                let is_z_not_zero = meta.query_advice(cols.z, Rotation::cur())
                    * meta.query_advice(cols.z_inv, Rotation::cur());
                // z_inv * z = 1 when z != 0
                cb.condition(is_z_not_zero.clone(), |cb| {
                    cb.require_equal(
                        "z_inv * z = 1",
                        meta.query_advice(cols.z, Rotation::cur())
                            * meta.query_advice(cols.z_inv, Rotation::cur()),
                        1.expr(),
                    );
                });
                // z_inv = 0 when z = 0
                cb.condition(not::expr(is_z_not_zero), |cb| {
                    cb.require_equal(
                        "z_inv = 0 when z = 0",
                        meta.query_advice(cols.z_inv, Rotation::cur()),
                        0.expr(),
                    );
                });
            }

            for (proj, p) in [
                (cin_sum_acc, cin_sum_acc_affine),
                (cout_sum_acc, cout_sum_acc_affine),
            ] {
                cb.require_equal(
                    "x = x * z_inv",
                    meta.query_advice(proj.x, Rotation::cur())
                        * meta.query_advice(proj.z_inv, Rotation::cur()),
                    meta.query_advice(p.x, Rotation::cur()),
                );
                cb.require_equal(
                    "y = y * z_inv",
                    meta.query_advice(proj.y, Rotation::cur())
                        * meta.query_advice(proj.z_inv, Rotation::cur()),
                    meta.query_advice(p.y, Rotation::cur()),
                );
            }

            cb.gate(meta.query_fixed(q_enable, Rotation::cur()))
        });

        Self {
            q_enable,
            is_first,
            is_last,
            inputs,
            cin_sum,
            cin_sum_acc,
            cin_sum_acc_affine,
            cout_sum,
            cout_sum_acc,
            cout_sum_acc_affine,
            pow2_table,
            escalarmul_config,
            encrypt_config,
            shuffle_config,
        }
    }
}

impl ShuffleEncryptCircuitConfig {
    pub fn assignments(
        &self,
        layouter: &mut impl Layouter<Fr>,
        n: usize,
        messages: &[(G1Affine, G1Affine)],
        encrypted: &[(G1Affine, G1Affine)],
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "shuffle encrypt",
            |mut region| {
                region.name_column(|| "q_enable", self.q_enable);
                region.name_column(|| "inputs", self.inputs);
                self.cin_sum.name_columns(&mut region, "cin_sum");
                self.cin_sum_acc.name_columns(&mut region, "cin_sum_acc");
                self.cin_sum_acc_affine
                    .name_columns(&mut region, "cin_sum_acc_affine");
                self.cout_sum.name_columns(&mut region, "cout_sum");
                self.cout_sum_acc.name_columns(&mut region, "cout_sum_acc");
                self.cout_sum_acc_affine
                    .name_columns(&mut region, "cout_sum_acc_affine");

                for index in 0..n {
                    region.assign_fixed(
                        || "q_enable",
                        self.q_enable,
                        index,
                        || Value::known(Fr::ONE),
                    )?;
                    region.assign_fixed(
                        || "is_first",
                        self.is_first,
                        index,
                        || Value::known(Fr::from(index == 0)),
                    )?;
                    region.assign_fixed(
                        || "is_last",
                        self.is_last,
                        index,
                        || Value::known(Fr::from(index == n - 1)),
                    )?;
                }

                let mut cin_sum_acc = G1::identity();
                let mut cout_sum_acc = G1::identity();
                for (index, (message, encrypted)) in
                    messages.iter().zip(encrypted.iter()).enumerate()
                {
                    let cin_sum = message.0 + message.1;
                    let cout_sum = encrypted.0 + encrypted.1;

                    self.cin_sum
                        .assign("cin_sum", &mut region, index, &cin_sum)?;
                    self.cout_sum
                        .assign("cout_sum", &mut region, index, &cout_sum)?;

                    if index == 0 {
                        cin_sum_acc = cin_sum; // add by identity will change the z
                        cout_sum_acc = cout_sum;
                    } else {
                        cin_sum_acc += cin_sum;
                        cout_sum_acc += cout_sum;
                    }

                    self.cin_sum_acc
                        .assign("cin_sum_acc", &mut region, index, &cin_sum_acc)?;
                    self.cout_sum_acc
                        .assign("cout_sum_acc", &mut region, index, &cout_sum_acc)?;

                    if index == n - 1 {
                        break; // leave assign with instance
                    } else {
                        self.cin_sum_acc_affine.assign(
                            "cin_sum_affine",
                            &mut region,
                            index,
                            &cin_sum_acc.to_affine(),
                        )?;
                        self.cout_sum_acc_affine.assign(
                            "cout_sum_affine",
                            &mut region,
                            index,
                            &cout_sum_acc.to_affine(),
                        )?;
                    }
                }

                // row = 1
                self.cin_sum_acc_affine.assign_from_instance(
                    "cin_sum_affine",
                    &mut region,
                    self.inputs,
                    0,
                    n - 1,
                )?;
                // row += 2
                self.cout_sum_acc_affine.assign_from_instance(
                    "cout_sum_affine",
                    &mut region,
                    self.inputs,
                    2,
                    n - 1,
                )?;

                Ok(())
            },
        )
    }
}

#[derive(Default)]
pub struct ShuffleEncryptCircuit<const N: usize>
where
    [(); N * 2]:,
{
    escalarmul_circuit: EscalarMulCircuit<{ N * 2 }>,
    encrypt_circuit: ElGamalEncryptCircuit<N>,
    shuffle_circuit: ShuffleCircuit<N>,
}

impl<const N: usize> SubCircuit for ShuffleEncryptCircuit<N>
where
    [(); N * 2]:,
{
    type Config = ShuffleEncryptCircuitConfig;

    fn synthesize_sub(
        &self,
        config: &Self::Config,
        layouter: &mut impl Layouter<Fr>,
    ) -> Result<(), Error> {
        config.assignments(
            layouter,
            N,
            &self.shuffle_circuit.messages,
            &self.shuffle_circuit.encrypted,
        )
    }
}

impl<const N: usize> Circuit<Fr> for ShuffleEncryptCircuit<N>
where
    [(); N * 2]:,
{
    type Config = ShuffleEncryptCircuitConfig;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        let pow2_table = Pow2Table::construct(meta);
        let escalarmul_table = EscalarMulTable::construct(meta);
        let encrypt_table = ElGamalEncryptTable::construct(meta);
        let shuffle_table = ShuffleTable::construct(meta);

        let args = ShuffleEncryptCircuitConfigArgs {
            pow2_table,
            escalarmul_config: EscalarMulCircuitConfig::new(
                meta,
                EscalarMulCircuitConfigArgs { escalarmul_table },
            ),
            encrypt_config: ElGamalEncryptCircuitConfig::new(
                meta,
                ElGamalEncryptCircuitConfigArgs {
                    encrypt_table,
                    escalarmul_table,
                },
            ),
            shuffle_config: ShuffleCircuitConfig::new(
                meta,
                ShuffleCircuitConfigArgs {
                    pow2_table,
                    encrypt_table,
                    shuffle_table,
                },
            ),
        };
        ShuffleEncryptCircuitConfig::new(meta, args)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        config.pow2_table.load(&mut layouter)?;
        self.escalarmul_circuit
            .synthesize_sub(&config.escalarmul_config, &mut layouter)?;
        self.encrypt_circuit
            .synthesize_sub(&config.encrypt_config, &mut layouter)?;
        self.shuffle_circuit
            .synthesize_sub(&config.shuffle_config, &mut layouter)?;
        self.synthesize_sub(&config, &mut layouter)?;

        Ok(())
    }
}
