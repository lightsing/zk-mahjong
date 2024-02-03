//! Chip that implements instructions to check: `a * b + c == d (mod p)`
//! where a, b, c, d, p are all 256-bit integers.
//! The modulus p is configured as circuit parameters.
//!
//! This is equivalent to `a * b + c - d - k * p == 0` where k is a 256-bit integer.
//!
//! Each Limb is 16 bits.

use halo2_proofs::plonk::{Advice, Column, Fixed};
use halo2curves::ff::{Field, PrimeField};
use halo2curves::goldilocks::Fp;
use std::array;

/// Config for the MulAdd Chip
#[derive(Clone, Debug)]
pub struct MulAddConfig {
    pub fixed_p: [Fp; 16],
    pub cols: [Column<Advice>; 16],
}
