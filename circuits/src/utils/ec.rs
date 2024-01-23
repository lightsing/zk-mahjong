use super::constraint_builder::BaseConstraintBuilder;
use crate::gadgets::utils::{select, Expr};
use ff::{Field, PrimeField};
use halo2_proofs::{
    circuit::{Region, Value},
    plonk::{
        Advice, Any, Column, ColumnType, ConstraintSystem, Error, Expression, Fixed, VirtualCells,
    },
};
use halo2curves::{
    bn256::Fr,
    grumpkin::{G1Affine, G1},
    CurveExt,
};

#[derive(Copy, Clone, Debug)]
pub struct ProjectivePointColumns {
    pub x: Column<Advice>,
    pub y: Column<Advice>,
    pub z: Column<Advice>,
    pub z_inv: Column<Advice>,
}

impl ProjectivePointColumns {
    pub fn construct(meta: &mut ConstraintSystem<Fr>) -> Self {
        let x = meta.advice_column();
        let y = meta.advice_column();
        let z = meta.advice_column();
        let z_inv = meta.advice_column();
        ProjectivePointColumns { x, y, z, z_inv }
    }

    pub fn assign(
        &self,
        point_name: &str,
        region: &mut Region<Fr>,
        offset: usize,
        point: &G1,
    ) -> Result<(), Error> {
        region.assign_advice(
            || format!("{point_name}.x at offset {}", offset),
            self.x,
            offset,
            || Value::known(point.x),
        )?;
        region.assign_advice(
            || format!("{point_name}.y at offset {}", offset),
            self.y,
            offset,
            || Value::known(point.y),
        )?;
        region.assign_advice(
            || format!("{point_name}.z at offset {}", offset),
            self.z,
            offset,
            || Value::known(point.z),
        )?;

        region.assign_advice(
            || format!("{point_name}.z_inv at offset {}", offset),
            self.z_inv,
            offset,
            || {
                if point.z.is_zero_vartime() {
                    Value::known(Fr::ZERO)
                } else {
                    Value::known(point.z.invert().unwrap())
                }
            },
        )?;
        Ok(())
    }

    pub fn name_columns(&self, region: &mut Region<'_, Fr>, name: &str) {
        region.name_column(|| format!("{}.x", name), self.x);
        region.name_column(|| format!("{}.y", name), self.y);
        region.name_column(|| format!("{}.z", name), self.z);
        region.name_column(|| format!("{}.z_inv", name), self.z_inv);
    }

    pub fn columns(&self) -> impl Iterator<Item = Column<Advice>> {
        [self.x, self.y, self.z].into_iter()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn constraint_add(
        meta: &mut VirtualCells<'_, Fr>,
        cb: &mut BaseConstraintBuilder<Fr>,
        lhs_x: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        lhs_y: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        lhs_z: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        rhs_x: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        rhs_y: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        rhs_z: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        result_x: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        result_y: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        result_z: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
    ) {
        // Algorithm 7, https://eprint.iacr.org/2015/1060.pdf
        let t0 = lhs_x(meta) * rhs_x(meta);
        let t1 = lhs_y(meta) * rhs_y(meta);
        let t2 = lhs_z(meta) * rhs_z(meta);
        let t3 = lhs_x(meta) + lhs_y(meta);
        let t4 = rhs_x(meta) + rhs_y(meta);
        let t3 = t3 * t4;
        let t4 = t0.clone() + t1.clone();
        let t3 = t3 - t4;
        let t4 = lhs_y(meta) + lhs_z(meta);
        let x3 = rhs_y(meta) + rhs_z(meta);
        let t4 = t4 * x3;
        let x3 = t1.clone() + t2.clone();
        let t4 = t4 - x3;
        let x3 = lhs_x(meta) + lhs_z(meta);
        let y3 = rhs_x(meta) + rhs_z(meta);
        let x3 = x3 * y3;
        let y3 = t0.clone() + t2.clone();
        let y3 = x3 - y3;
        let x3 = t0.clone() + t0.clone();
        let t0 = x3 + t0;
        let t2 = 3.expr() * Expression::Constant(G1::b()) * t2;
        let z3 = t1.clone() + t2.clone();
        let t1 = t1.clone() - t2;
        let y3 = 3.expr() * Expression::Constant(G1::b()) * y3.clone();
        let x3 = t4.clone() * y3.clone();
        let t2 = t3.clone() * t1.clone();
        let x3 = t2 - x3;
        let y3 = y3 * t0.clone();
        let t1 = t1 * z3.clone();
        let y3 = t1 + y3;
        let t0 = t0 * t3;
        let z3 = z3 * t4;
        let z3 = z3 + t0;

        cb.require_equal("addition of points", result_x(meta), x3);
        cb.require_equal("addition of points", result_y(meta), y3);
        cb.require_equal("addition of points", result_z(meta), z3);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn constraint_double(
        meta: &mut VirtualCells<'_, Fr>,
        cb: &mut BaseConstraintBuilder<Fr>,
        x: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        y: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        z: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        z_inv: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        result_x: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        result_y: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
        result_z: impl Fn(&mut VirtualCells<'_, Fr>) -> Expression<Fr>,
    ) {
        // Algorithm 9, https://eprint.iacr.org/2015/1060.pdf
        let t0 = y(meta) * y(meta);
        let z3 = 8.expr() * t0.clone();
        let t1 = y(meta) * z(meta);
        let t2 = z(meta) * z(meta);
        let t2 = 3.expr() * Expression::Constant(G1::b()) * t2;
        let x3 = t2.clone() * z3.clone();
        let y3 = t0.clone() + t2.clone();
        let z3 = t1.clone() * z3.clone();
        let t2 = 3.expr() * t2;
        let t0 = t0.clone() - t2.clone();
        let y3 = t0.clone() * y3;
        let y3 = x3.clone() + y3;
        let t1 = x(meta) * y(meta);
        let x3 = 2.expr() * t0 * t1;

        let is_z_zero = 1.expr() - z(meta) * z_inv(meta);

        cb.require_equal(
            "double of point",
            result_x(meta),
            select::expr(is_z_zero.clone(), 0.expr(), x3),
        );
        cb.require_equal(
            "double of point",
            result_y(meta),
            select::expr(is_z_zero.clone(), 1.expr(), y3),
        );
        cb.require_equal(
            "double of point",
            result_z(meta),
            select::expr(is_z_zero, 0.expr(), z3),
        );
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PointColumns<C: ColumnType> {
    pub x: Column<C>,
    pub y: Column<C>,
}

impl<C> PointColumns<C>
where
    C: ColumnType,
    Column<Any>: From<Column<C>>,
{
    pub fn name_columns(&self, region: &mut Region<'_, Fr>, name: &str) {
        region.name_column(|| format!("{}.x", name), self.x);
        region.name_column(|| format!("{}.y", name), self.y);
    }

    pub fn columns(&self) -> impl Iterator<Item = Column<C>> {
        [self.x, self.y].into_iter()
    }
}

impl PointColumns<Advice> {
    pub fn construct<F: PrimeField>(meta: &mut ConstraintSystem<F>) -> Self {
        let x = meta.advice_column();
        let y = meta.advice_column();
        PointColumns { x, y }
    }

    pub fn assign(
        &self,
        point_name: &str,
        region: &mut Region<Fr>,
        offset: usize,
        point: &G1Affine,
    ) -> Result<(), Error> {
        region.assign_advice(
            || format!("{point_name}.x at offset {}", offset),
            self.x,
            offset,
            || Value::known(point.x),
        )?;
        region.assign_advice(
            || format!("{point_name}.y at offset {}", offset),
            self.y,
            offset,
            || Value::known(point.y),
        )?;
        Ok(())
    }
}

impl PointColumns<Fixed> {
    pub fn construct<F: PrimeField>(meta: &mut ConstraintSystem<F>) -> Self {
        let x = meta.fixed_column();
        let y = meta.fixed_column();
        PointColumns { x, y }
    }

    pub fn assign(
        &self,
        point_name: &str,
        region: &mut Region<Fr>,
        offset: usize,
        point: &G1Affine,
    ) -> Result<(), Error> {
        region.assign_fixed(
            || format!("{point_name}.x at offset {}", offset),
            self.x,
            offset,
            || Value::known(point.x),
        )?;
        region.assign_fixed(
            || format!("{point_name}.y at offset {}", offset),
            self.y,
            offset,
            || Value::known(point.y),
        )?;
        Ok(())
    }
}
