use crate::{
    gadgets::utils::{and, not, Expr},
    sub_circuits::{
        tables::escalarmul::{EscalarMulAssignRow, EscalarMulTable},
        SubCircuit, SubCircuitConfig,
    },
    utils::{constraint_builder::BaseConstraintBuilder, ec::ProjectivePointColumns},
};
use ff::Field;
use halo2_proofs::{
    circuit::{Layouter, Region, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};
use halo2curves::{bn256::Fr, group::Curve, grumpkin::G1Affine};

#[cfg(test)]
mod dev;
#[cfg(test)]
mod test;

#[derive(Copy, Clone, Debug)]
pub struct EscalarMulCircuitConfig {
    /// Whether the row is enabled.
    pub q_enable: Column<Fixed>,
    /// Whether this row is the first row in the EC Scalar Mul's trace.
    pub is_first: Column<Advice>,
    /// bit decomposition of the scalar
    pub scalar_bit: Column<Advice>,
    /// result of the scalar multiplication
    pub result: ProjectivePointColumns,
    /// result of the scalar multiplication * 2
    pub result2: ProjectivePointColumns,
    /// lookup table
    pub escalarmul_table: EscalarMulTable,
}

pub struct EscalarMulCircuitConfigArgs {
    /// lookup table
    pub escalarmul_table: EscalarMulTable,
}

impl SubCircuitConfig for EscalarMulCircuitConfig {
    type ConfigArgs = EscalarMulCircuitConfigArgs;

    fn new(
        meta: &mut ConstraintSystem<Fr>,
        EscalarMulCircuitConfigArgs { escalarmul_table }: Self::ConfigArgs,
    ) -> Self {
        let q_enable = escalarmul_table.q_enable;
        let is_first = meta.advice_column();
        let scalar_bit = meta.advice_column();
        let result = ProjectivePointColumns::construct(meta);
        let result2 = ProjectivePointColumns::construct(meta);

        meta.create_gate("verify all rows", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            cb.require_boolean(
                "scalar_bit is either 0 or 1",
                meta.query_advice(scalar_bit, Rotation::cur()),
            );

            for cols in [result, result2] {
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

            {
                let (projective, point) = (result, escalarmul_table.result);
                cb.require_equal(
                    "x = x * z_inv",
                    meta.query_advice(projective.x, Rotation::cur())
                        * meta.query_advice(projective.z_inv, Rotation::cur()),
                    meta.query_advice(point.x, Rotation::cur()),
                );
                cb.require_equal(
                    "y = y * z_inv",
                    meta.query_advice(projective.y, Rotation::cur())
                        * meta.query_advice(projective.z_inv, Rotation::cur()),
                    meta.query_advice(point.y, Rotation::cur()),
                );
            }

            // result2 = result * 2
            ProjectivePointColumns::constraint_double(
                meta,
                &mut cb,
                |meta| meta.query_advice(result.x, Rotation::cur()),
                |meta| meta.query_advice(result.y, Rotation::cur()),
                |meta| meta.query_advice(result.z, Rotation::cur()),
                |meta| meta.query_advice(result.z_inv, Rotation::cur()),
                |meta| meta.query_advice(result2.x, Rotation::cur()),
                |meta| meta.query_advice(result2.y, Rotation::cur()),
                |meta| meta.query_advice(result2.z, Rotation::cur()),
            );

            cb.gate(meta.query_fixed(q_enable, Rotation::cur()))
        });

        meta.create_gate("verify first row", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            cb.require_equal(
                "scalar_acc inital as scalar_bit",
                meta.query_advice(scalar_bit, Rotation::cur()),
                meta.query_advice(escalarmul_table.scalar_acc, Rotation::cur()),
            );

            // result starts as (0, 1, 0) = identity
            for (col, val) in result.columns().zip([0, 1, 0]) {
                cb.require_equal(
                    "result starts as (0, 1, 0)",
                    meta.query_advice(col, Rotation::cur()),
                    val.expr(),
                );
            }

            cb.gate(and::expr([
                meta.query_fixed(q_enable, Rotation::cur()),
                meta.query_advice(is_first, Rotation::cur()),
            ]))
        });

        meta.create_gate("verify last row", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            cb.require_zero(
                "scalar_bit == 0",
                meta.query_advice(scalar_bit, Rotation::cur()),
            );

            // scalar_acc::cur = scalar_acc::prev
            cb.require_equal(
                "scalar_acc::cur = scalar_acc::prev",
                meta.query_advice(escalarmul_table.scalar_acc, Rotation::cur()),
                meta.query_advice(escalarmul_table.scalar_acc, Rotation::prev()),
            );

            cb.gate(and::expr([
                meta.query_fixed(q_enable, Rotation::cur()),
                meta.query_advice(escalarmul_table.is_last, Rotation::cur()),
            ]))
        });

        meta.create_gate("verify all rows but last", |meta| {
            let mut cb = BaseConstraintBuilder::default();

            for cols in escalarmul_table.base.columns() {
                cb.require_equal(
                    "base::cur = base::next",
                    meta.query_advice(cols, Rotation::cur()),
                    meta.query_advice(cols, Rotation::next()),
                );
            }

            cb.condition(
                not::expr(meta.query_advice(escalarmul_table.is_last, Rotation::next())),
                |cb| {
                    // scalar_acc::next = scalar_acc::cur * 2 + scalar_bit::next
                    cb.require_equal(
                        "scalar_acc::next = scalar_acc::cur * 2 + scalar_bit::next",
                        meta.query_advice(escalarmul_table.scalar_acc, Rotation::next()),
                        meta.query_advice(escalarmul_table.scalar_acc, Rotation::cur()) * 2.expr()
                            + meta.query_advice(scalar_bit, Rotation::next()),
                    );
                },
            );

            cb.condition(meta.query_advice(scalar_bit, Rotation::cur()), |cb| {
                // result::next = result2::cur + base::cur
                ProjectivePointColumns::constraint_add(
                    meta,
                    cb,
                    |meta| meta.query_advice(result2.x, Rotation::cur()),
                    |meta| meta.query_advice(result2.y, Rotation::cur()),
                    |meta| meta.query_advice(result2.z, Rotation::cur()),
                    |meta| meta.query_advice(escalarmul_table.base.x, Rotation::cur()),
                    |meta| meta.query_advice(escalarmul_table.base.y, Rotation::cur()),
                    |_| 1.expr(),
                    |meta| meta.query_advice(result.x, Rotation::next()),
                    |meta| meta.query_advice(result.y, Rotation::next()),
                    |meta| meta.query_advice(result.z, Rotation::next()),
                );
            });

            cb.condition(
                not::expr(meta.query_advice(scalar_bit, Rotation::cur())),
                |cb| {
                    // result::next = result2::cur
                    for (result_col, result2_col) in result.columns().zip(result2.columns()) {
                        cb.require_equal(
                            "result::next = result2::cur",
                            meta.query_advice(result_col, Rotation::next()),
                            meta.query_advice(result2_col, Rotation::cur()),
                        );
                    }
                },
            );

            cb.gate(and::expr([
                meta.query_fixed(q_enable, Rotation::cur()),
                not::expr(meta.query_advice(escalarmul_table.is_last, Rotation::cur())),
            ]))
        });

        EscalarMulCircuitConfig {
            q_enable,
            is_first,
            scalar_bit,
            result,
            result2,
            escalarmul_table,
        }
    }
}

impl EscalarMulCircuitConfig {
    pub fn assign_scalar_muls(
        &self,
        layouter: &mut impl Layouter<Fr>,
        max_muls: usize,
        muls: &[(G1Affine, Fr)],
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "exponentiation circuit",
            |mut region| {
                region.name_column(|| "q_enable", self.q_enable);
                region.name_column(|| "is_first", self.is_first);
                region.name_column(|| "is_last", self.escalarmul_table.is_last);
                region.name_column(|| "scalar_bit", self.scalar_bit);
                region.name_column(|| "scalar_acc", self.escalarmul_table.scalar_acc);
                self.result.name_columns(&mut region, "result");
                self.result2.name_columns(&mut region, "result");
                self.escalarmul_table.base.name_columns(&mut region, "base");
                self.escalarmul_table
                    .result
                    .name_columns(&mut region, "result");

                let mut offset = 0;
                for (base, scalar) in muls.iter() {
                    self.assign_scalar_mul(&mut region, &mut offset, *base, *scalar)?;
                }

                // Pad the rest of the table with zeros.
                for _ in muls.len()..max_muls {
                    self.assign_scalar_mul(
                        &mut region,
                        &mut offset,
                        G1Affine::generator(),
                        Fr::ONE,
                    )?;
                }

                Ok(())
            },
        )
    }

    fn assign_scalar_mul(
        &self,
        region: &mut Region<Fr>,
        offset: &mut usize,
        base: G1Affine,
        scalar: Fr,
    ) -> Result<(), Error> {
        let assignments = EscalarMulTable::assign(base, scalar);
        let assignments_len = assignments.len();
        for (
            idx,
            EscalarMulAssignRow {
                scalar_bit,
                scalar_acc,
                result,
                result2,
            },
        ) in assignments.into_iter().enumerate()
        {
            let is_last = idx == assignments_len - 1;

            region.assign_fixed(
                || format!("q_enable at {offset}"),
                self.q_enable,
                *offset,
                || Value::known(Fr::ONE),
            )?;

            region.assign_advice(
                || format!("is_first at {offset}"),
                self.is_first,
                *offset,
                || Value::known(Fr::from((idx == 0) as u64)),
            )?;
            region.assign_advice(
                || format!("is_last at {offset}"),
                self.escalarmul_table.is_last,
                *offset,
                || Value::known(Fr::from(is_last as u64)),
            )?;

            region.assign_advice(
                || format!("scalar_bit[{idx}] at {offset}"),
                self.scalar_bit,
                *offset,
                || Value::known(scalar_bit),
            )?;
            region.assign_advice(
                || format!("scalar_acc at {offset}"),
                self.escalarmul_table.scalar_acc,
                *offset,
                || Value::known(scalar_acc),
            )?;
            self.result.assign("result", region, *offset, &result)?;
            self.result2.assign("result", region, *offset, &result2)?;
            self.escalarmul_table
                .base
                .assign("base", region, *offset, &base)?;
            self.escalarmul_table
                .result
                .assign("result", region, *offset, &result.to_affine())?;
            *offset += 1;
        }

        Ok(())
    }
}

#[derive(Default, Clone, Debug)]
pub struct EscalarMulCircuit<const MAX_MULS: usize> {
    /// Multiplications
    pub muls: Vec<(G1Affine, Fr)>,
}

impl<const MAX_MULS: usize> EscalarMulCircuit<MAX_MULS> {
    pub fn new(muls: Vec<(G1Affine, Fr)>) -> Self {
        EscalarMulCircuit { muls }
    }
}

impl<const MAX_MULS: usize> SubCircuit for EscalarMulCircuit<MAX_MULS> {
    type Config = EscalarMulCircuitConfig;

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
        config.assign_scalar_muls(layouter, MAX_MULS, &self.muls)
    }
}
