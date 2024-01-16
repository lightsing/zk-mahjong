use core::fmt::Display;
use std::hash::Hash;
use ff::*;
use num_bigint::BigUint;
use serde::{Serialize, Deserialize};

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
#[derive(Hash)]
pub struct Fr([u64; 4]);

impl phf_shared::PhfHash for Fr {
    fn phf_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.phf_hash(state)
    }
}

impl phf_shared::FmtConst for Fr {
    fn fmt_const(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "crate::bn128::Fr::from_raw([")?;
        for i in 0..4 {
            write!(f, "0x{:016x}, ", self.0[i])?;
        }
        write!(f, "])")
    }
}

impl phf_shared::PhfBorrow<Fr> for Fr {
    fn borrow(&self) -> &Fr {
        &self
    }
}

impl Fr {
    pub const TWO: Fr = Fr::from_raw([0x592c68389ffffff6, 0x6df8ed2b3ec19a53, 0xccdd46def0f28c5c, 0x1c14ef83340fbe5e]);
    
    pub(crate) const fn from_raw(val: [u64; 4]) -> Self {
        Fr(val)
    }

    pub const fn into_raw(self) -> [u64; 4] {
        self.0
    }

    pub fn as_raw(&self) -> &[u64; 4] {
        &self.0
    }

    pub fn to_bigint(&self) -> BigUint {
        let repr = self.to_repr().0;
        BigUint::from_bytes_le(&repr)
    }
}

impl Display for Fr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Serialize for Fr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let big = self.to_bigint();
        let mut s = big.to_str_radix(10);
        s.push('n');
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Fr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Fr::from_str_vartime(s[..s.len()-1].as_ref())
            .ok_or_else(|| serde::de::Error::custom("invalid field element"))
    }
}