use crate::{
    gadgets::utils::{not, Expr},
    tables::{
        encrypt::{ElGamalEncryptAssignRow, ElGamalEncryptTable},
        escalarmul::EscalarMulTable,
        LookupTable,
    },
    utils::{
        constraint_builder::BaseConstraintBuilder,
        ec::{PointColumns, ProjectivePointColumns},
        SubCircuit, SubCircuitConfig,
    },
};
use ff::Field;
use halo2_proofs::{
    circuit::{Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};
use halo2curves::{bn256::Fr, group::Curve};
use halo2curves::{group::cofactor::CofactorCurveAffine, grumpkin::G1Affine};

#[cfg(test)]
mod dev;
#[cfg(test)]
mod test;

#[derive(Copy, Clone, Debug)]
pub struct ElGamalEncryptCircuitConfig {
    pub q_enable: Column<Fixed>,
    /// randomness
    pub r: Column<Advice>,
    /// r * G
    pub r_g: PointColumns<Advice>,
    /// r * H
    pub r_h: PointColumns<Advice>,
    /// [r * G + c0, r * H + c1]
    pub cin: [ProjectivePointColumns; 2],
    pub cout: [ProjectivePointColumns; 2],
    pub encrypt_table: ElGamalEncryptTable,
    pub escalarmul_table: EscalarMulTable,
}

#[derive()]
pub struct ElGamalEncryptCircuitConfigArgs {
    /// lookup table
    pub encrypt_table: ElGamalEncryptTable,
    /// lookup table
    pub escalarmul_table: EscalarMulTable,
}

impl SubCircuitConfig for ElGamalEncryptCircuitConfig {
    type ConfigArgs = ElGamalEncryptCircuitConfigArgs;

    fn new(
        meta: &mut ConstraintSystem<Fr>,
        ElGamalEncryptCircuitConfigArgs {
            encrypt_table,
            escalarmul_table,
        }: Self::ConfigArgs,
    ) -> Self {
        let q_enable = encrypt_table.q_enable;
        let r = meta.advice_column();
        let r_g = PointColumns::<Advice>::construct(meta);
        let r_h = PointColumns::<Advice>::construct(meta);
        let cin = [
            ProjectivePointColumns::construct(meta),
            ProjectivePointColumns::construct(meta),
        ];
        let cout = [
            ProjectivePointColumns::construct(meta),
            ProjectivePointColumns::construct(meta),
        ];

        let g = G1Affine::generator();

        meta.lookup_any("lookup rG", |meta| {
            let q_enable = meta.query_fixed(q_enable, Rotation::cur());
            [
                1.expr(),
                1.expr(),
                meta.query_advice(r, Rotation::cur()),
                Expression::Constant(g.x),
                Expression::Constant(g.y),
                meta.query_advice(r_g.x, Rotation::cur()),
                meta.query_advice(r_g.y, Rotation::cur()),
            ]
            .into_iter()
            .zip(escalarmul_table.table_exprs(meta))
            .map(|(arg, table)| (q_enable.clone() * arg, table))
            .collect()
        });

        meta.lookup_any("lookup rH", |meta| {
            let q_enable = meta.query_fixed(q_enable, Rotation::cur());
            [
                1.expr(),
                1.expr(),
                meta.query_advice(r, Rotation::cur()),
                meta.query_advice(encrypt_table.agg_pk.x, Rotation::cur()),
                meta.query_advice(encrypt_table.agg_pk.y, Rotation::cur()),
                meta.query_advice(r_h.x, Rotation::cur()),
                meta.query_advice(r_h.y, Rotation::cur()),
            ]
            .into_iter()
            .zip(escalarmul_table.table_exprs(meta))
            .map(|(arg, table)| (q_enable.clone() * arg, table))
            .collect()
        });

        meta.create_gate("verify all rows", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            cb.require_boolean(
                "cin[0].z == 0 | 1",
                meta.query_advice(cin[0].z, Rotation::cur()),
            );
            cb.require_equal(
                "cin[1].z == 1",
                meta.query_advice(cin[1].z, Rotation::cur()),
                Expression::Constant(Fr::ONE),
            );

            for cols in cin.iter().chain(cout.iter()) {
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

            // cout holds
            for (proj, p) in cin
                .iter()
                .zip(encrypt_table.cin)
                .chain(cout.iter().zip(encrypt_table.cout))
            {
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

            for (lhs, rhs, result) in [(r_g, cin[0], cout[0]), (r_h, cin[1], cout[1])] {
                ProjectivePointColumns::constraint_add(
                    meta,
                    &mut cb,
                    |meta| meta.query_advice(lhs.x, Rotation::cur()),
                    |meta| meta.query_advice(lhs.y, Rotation::cur()),
                    |_| Expression::Constant(Fr::ONE),
                    |meta| meta.query_advice(rhs.x, Rotation::cur()),
                    |meta| meta.query_advice(rhs.y, Rotation::cur()),
                    |meta| meta.query_advice(rhs.z, Rotation::cur()),
                    |meta| meta.query_advice(result.x, Rotation::cur()),
                    |meta| meta.query_advice(result.y, Rotation::cur()),
                    |meta| meta.query_advice(result.z, Rotation::cur()),
                );
            }

            cb.gate(meta.query_fixed(q_enable, Rotation::cur()))
        });

        ElGamalEncryptCircuitConfig {
            q_enable,
            r,
            r_g,
            r_h,
            cin,
            cout,
            encrypt_table,
            escalarmul_table,
        }
    }
}

impl ElGamalEncryptCircuitConfig {
    pub fn assign_messages(
        &self,
        layouter: &mut impl Layouter<Fr>,
        agg_pk: &G1Affine,
        r: &[Fr],
        messages: &[(G1Affine, G1Affine)],
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "elgamal encrypt",
            |mut region| {
                region.name_column(|| "q_enable", self.q_enable);
                region.name_column(|| "r", self.r);
                self.r_g.name_columns(&mut region, "rG");
                self.r_h.name_columns(&mut region, "rH");
                self.cout[0].name_columns(&mut region, "c_out0_projective");
                self.cout[1].name_columns(&mut region, "c_out1_projective");
                self.encrypt_table
                    .agg_pk
                    .name_columns(&mut region, "agg_pk");
                region.name_column(|| "index", self.encrypt_table.index);
                self.encrypt_table.cin[0].name_columns(&mut region, "c_in0");
                self.encrypt_table.cin[1].name_columns(&mut region, "c_in1");
                self.encrypt_table.cout[0].name_columns(&mut region, "c_out0");
                self.encrypt_table.cout[1].name_columns(&mut region, "c_out1");

                let assignments = ElGamalEncryptTable::assignments(agg_pk, r, messages);
                for ElGamalEncryptAssignRow {
                    index,
                    r_g,
                    r_h,
                    cout0,
                    cout1,
                } in assignments
                {
                    region.assign_fixed(
                        || "q_enable",
                        self.q_enable,
                        index,
                        || Value::known(Fr::ONE),
                    )?;
                    self.encrypt_table.generator.assign(
                        "generator",
                        &mut region,
                        index,
                        &G1Affine::generator(),
                    )?;
                    self.encrypt_table
                        .agg_pk
                        .assign("agg_pk", &mut region, index, agg_pk)?;
                    region.assign_advice(
                        || "index",
                        self.encrypt_table.index,
                        index,
                        || Value::known(Fr::from(index as u64)),
                    )?;
                    self.encrypt_table.cin[0].assign(
                        "c_in",
                        &mut region,
                        index,
                        &messages[index].0,
                    )?;
                    self.encrypt_table.cin[1].assign(
                        "c_in",
                        &mut region,
                        index,
                        &messages[index].1,
                    )?;
                    self.encrypt_table.cout[0].assign(
                        "c_out",
                        &mut region,
                        index,
                        &cout0.to_affine(),
                    )?;
                    self.encrypt_table.cout[1].assign(
                        "c_out",
                        &mut region,
                        index,
                        &cout1.to_affine(),
                    )?;
                    region.assign_advice(|| "r", self.r, index, || Value::known(r[index]))?;
                    self.r_g.assign("r_g", &mut region, index, &r_g)?;
                    self.r_h.assign("r_h", &mut region, index, &r_h)?;
                    self.cin[0].assign(
                        "c_in0",
                        &mut region,
                        index,
                        &messages[index].0.to_curve(),
                    )?;
                    self.cin[1].assign(
                        "c_in0",
                        &mut region,
                        index,
                        &messages[index].1.to_curve(),
                    )?;
                    self.cout[0].assign("c_out", &mut region, index, &cout0)?;
                    self.cout[1].assign("c_out", &mut region, index, &cout1)?;
                }
                Ok(())
            },
        )
    }
}

#[derive(Default, Clone, Debug)]
pub struct ElGamalEncryptCircuit {
    /// aggregate public key
    pub agg_pk: G1Affine,
    /// randomness
    pub r: Vec<Fr>,
    pub messages: Vec<(G1Affine, G1Affine)>,
}

impl ElGamalEncryptCircuit {
    pub fn new(agg_pk: G1Affine, r: Vec<Fr>, messages: Vec<(G1Affine, G1Affine)>) -> Self {
        Self {
            agg_pk,
            r,
            messages,
        }
    }
}

impl SubCircuit for ElGamalEncryptCircuit {
    type Config = ElGamalEncryptCircuitConfig;

    fn unusable_rows() -> usize {
        // No column queried at more than 3 distinct rotations, so returns 6 as
        // minimum unusable rows.
        6
    }

    fn synthesize_sub(
        &self,
        config: &Self::Config,
        layouter: &mut impl Layouter<Fr>,
    ) -> Result<(), Error> {
        config.assign_messages(layouter, &self.agg_pk, &self.r, &self.messages)
    }
}
