use super::LookupTable;
use crate::utils::ec::PointColumns;
use halo2_proofs::plonk::{Advice, Any, Column, ConstraintSystem, Fixed};
use halo2curves::{
    bn256::{Fq, Fr},
    group::Curve,
    grumpkin::{G1Affine, G1},
};
use smallvec::smallvec;

/// Lookup table within the ElGamalEncrypt circuit.
#[derive(Clone, Copy, Debug)]
pub struct ElGamalEncryptTable {
    /// Whether the row is enabled.
    pub q_enable: Column<Fixed>,
    /// The index of the row.
    pub index: Column<Fixed>,
    /// The aggregate public key.
    pub agg_pk: PointColumns<Advice>,
    /// The message to encrypt.
    pub cin: [PointColumns<Advice>; 2],
    /// The encrypted message.
    pub cout: [PointColumns<Advice>; 2],
}

pub struct ElGamalEncryptAssignRow {
    pub index: usize,
    pub r_g: G1Affine,
    pub r_h: G1Affine,
    pub cout0: G1,
    pub cout1: G1,
}

impl ElGamalEncryptTable {
    /// Construct the ElGamal Encrypt table.
    pub fn construct(meta: &mut ConstraintSystem<Fr>) -> Self {
        let q_enable = meta.fixed_column();
        let index = meta.fixed_column();
        let agg_pk = PointColumns::<Advice>::construct(meta);
        let cin = [
            PointColumns::<Advice>::construct(meta),
            PointColumns::<Advice>::construct(meta),
        ];
        let cout = [
            PointColumns::<Advice>::construct(meta),
            PointColumns::<Advice>::construct(meta),
        ];
        ElGamalEncryptTable {
            q_enable,
            agg_pk,
            index,
            cin,
            cout,
        }
    }

    pub fn assignments(
        agg_pk: &G1Affine,
        r: &[Fr],
        message: &[(G1Affine, G1Affine)],
    ) -> Vec<ElGamalEncryptAssignRow> {
        assert_eq!(r.len(), message.len());
        let mut assignment = Vec::with_capacity(r.len());
        for (i, (r, (c0, c1))) in r.iter().zip(message.iter()).enumerate() {
            let r = Fq::from_bytes(&r.to_bytes()).unwrap();
            let r_g = (G1Affine::generator() * r).to_affine();
            let r_h = (agg_pk * r).to_affine();
            assignment.push(ElGamalEncryptAssignRow {
                index: i,
                r_g,
                r_h,
                cout0: r_g + c0,
                cout1: r_h + c1,
            });
        }
        assignment
    }
}

impl LookupTable for ElGamalEncryptTable {
    fn columns(&self) -> super::Columns<Any> {
        smallvec![
            self.q_enable.into(),
            self.agg_pk.x.into(),
            self.agg_pk.y.into(),
            self.index.into(),
            self.cin[0].x.into(),
            self.cin[0].y.into(),
            self.cin[1].x.into(),
            self.cin[1].y.into(),
            self.cout[0].x.into(),
            self.cout[0].y.into(),
            self.cout[1].x.into(),
            self.cout[1].y.into(),
        ]
    }

    fn annotations(&self) -> super::Annotations {
        smallvec![
            "q_enable".into(),
            "agg_pk.x".into(),
            "agg_pk.y".into(),
            "index".into(),
            "cin[0].x".into(),
            "cin[0].y".into(),
            "cin[1].x".into(),
            "cin[1].y".into(),
            "cout[0].x".into(),
            "cout[0].y".into(),
            "cout[1].x".into(),
            "cout[1].y".into(),
        ]
    }
}
