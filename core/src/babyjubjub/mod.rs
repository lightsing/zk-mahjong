use std::ops::{Add, AddAssign, Deref, Neg, Sub};

use crate::bn128::{Fr, FrRepr};
use ff::{
    derive::{
        bitvec::{view::AsBits, order::Lsb0},
        subtle::*,
    },
    Field, PrimeField,
};
use constants::*;
pub use constants::BASE_POINT;
use serde::{Serialize, Deserialize};

mod constants;
#[cfg(test)]
mod tests;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: Fr,
    pub y: Fr,
}

impl ConditionallySelectable for Point {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Point {
            x: Fr::conditional_select(&a.x, &b.x, choice),
            y: Fr::conditional_select(&a.y, &b.y, choice),
        }
    }
}

impl Add for Point {
    type Output = PointProjective;

    fn add(self, other: Point) -> PointProjective {
        // https://hyperelliptic.org/EFD/g1p/auto-twisted-projective.html#addition-mmadd-2008-bbjlp
        let c = self.x * other.x;
        let d = self.y * other.y;
        let e = D * c * d;
        
        PointProjective {
            x: (Fr::ONE - e) * ((self.x + self.y) * (other.x + other.y) - c - d),
            y: (Fr::ONE + e) * (d - A * c),
            z: Fr::ONE - e.square(),
        }
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Point {
        Point {
            x: Fr::ZERO - self.x,
            y: self.y,
        }
    }
}

impl Sub for Point {
    type Output = PointProjective;

    fn sub(self, other: Point) -> PointProjective {
        self + (-other)
    }
}

impl Point {
    const O: Point = Point {
        x: Fr::ZERO,
        y: Fr::ONE,
    };
}

impl Point {
    pub const ZERO: Point = Point {
        x: Fr::ZERO,
        y: Fr::ONE,
    };
    pub const BASE_POINT: Point = BASE_POINT;

    pub fn new(x: Fr, y: Fr) -> Point {
        Point { x, y }
    }

    pub fn projective(&self) -> PointProjective {
        PointProjective {
            x: self.x,
            y: self.y,
            z: Fr::ONE,
        }
    }

    pub fn mul_scalar(&self, n: &Fr) -> Point {
        let mut r = PointProjective {
            x: Fr::ZERO,
            y: Fr::ONE,
            z: Fr::ONE,
        };
        let mut exp: PointProjective = self.projective();
        let repr = n.to_repr();
        let bits = repr.as_bits::<Lsb0>();
        let n_bits = bits.len() - bits.trailing_zeros();
        for i in 0..n_bits {
            let tmp = r + exp;
            r.conditional_assign(&tmp, Choice::from(bits[i] as u8));
            exp = exp.double();
        }
        r.affine()
    }

    pub fn compress(&self) -> [u8; 32] {
        let mut repr = self.y.to_repr().0;
        let sign = repr[31] | 0x80;
        repr[31].conditional_assign(&sign, Choice::from((self.x > Q_SHR_1) as u8));
        repr
    }

    pub fn decompress(mut buf: [u8; 32]) -> CtOption<Self> {
        let sign = Choice::from((buf[31] & 0x80 != 0) as u8);
        buf[31].conditional_assign(&(buf[31] & 0x7f), sign);
        Fr::from_repr(FrRepr(buf)).and_then(|y| {
            // x^2 = (1 - y^2) / (a - d * y^2) (mod p)
            let y2 = y.square();
            (A - D * y2).invert().and_then(|denominator| {
                let x2 = (Fr::ONE - y2) * denominator;
                x2.sqrt().map(|mut x| {
                    // if sign && (x <= (&Q.clone() >> 1)) || (!sign && (x > (&Q.clone() >> 1))) {
                    //     x *= -(1.to_bigint().unwrap());
                    // }
                    let choice_lt = Choice::from((x <= Q_SHR_1) as u8);
                    let choice_ge = Choice::from((x > Q_SHR_1) as u8);
                    let choice = sign & choice_lt | (!sign) & choice_ge;
                    x.conditional_assign(&x.neg(), choice);
                    Point { x, y }
                })
            })
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PointProjective {
    pub x: Fr,
    pub y: Fr,
    pub z: Fr,
}

impl ConditionallySelectable for PointProjective {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        PointProjective {
            x: Fr::conditional_select(&a.x, &b.x, choice),
            y: Fr::conditional_select(&a.y, &b.y, choice),
            z: Fr::conditional_select(&a.z, &b.z, choice),
        }
    }
}

impl PointProjective {
    pub fn affine(&self) -> Point {
        self.z
            .invert()
            .map(|zinv| Point {
                x: self.x * zinv,
                y: self.y * zinv,
            })
            .unwrap_or(Point {
                x: Fr::ZERO,
                y: Fr::ZERO,
            })
    }

    pub fn add(&self, other: &PointProjective) -> PointProjective {
        *self + *other
    }

    pub fn double(&self) -> PointProjective {
        // https://hyperelliptic.org/EFD/g1p/auto-twisted-projective.html#doubling-dbl-2008-bbjlp
        let b = (self.x + self.y).square();
        let c = self.x.square();
        let d = self.y.square();
        let e = A * c;
        let f = e + d;
        let h = self.z.square();
        let j = f - Fr::TWO * h;
        PointProjective {
            x: (b - c - d) * j,
            y: f * (e - d),
            z: f * j,
        }
    }
}

impl Add for PointProjective {
    type Output = PointProjective;

    fn add(self, other: PointProjective) -> PointProjective {
        // https://hyperelliptic.org/EFD/g1p/auto-twisted-projective.html#addition-add-2008-bbjlp
        let a = self.z * other.z;
        let b = a.square();
        let c = self.x * other.x;
        let d = self.y * other.y;
        let e = D * c * d;
        let f = b - e;
        let g = b + e;
        PointProjective {
            x: a * f * ((self.x + self.y) * (other.x + other.y) - c - d),
            y: a * g * (d - A * c),
            z: f * g,
        }
    }
}

impl Add<Point> for PointProjective {
    type Output = PointProjective;

    fn add(self, other: Point) -> PointProjective {
        self + other.projective()
    }
}

impl AddAssign for PointProjective {
    fn add_assign(&mut self, other: PointProjective) {
        *self = *self + other;
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct SecretKey(pub(crate) Fr);

impl SecretKey {
    pub fn random() -> SecretKey {
        let mut rng = rand::thread_rng();
        loop {
            let fr = Fr::random(&mut rng);
            if fr.is_zero_vartime() {
                continue;
            }
            return SecretKey(fr);
        }
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(BASE_POINT.mul_scalar(&self.0))
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct PublicKey(Point);

impl Deref for PublicKey {
    type Target = Point;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PublicKey {
    pub fn aggregate(others: Vec<PublicKey>) -> PublicKey {
        assert_eq!(others.len(), 4);
        let a = others[0].0 + others[1].0;
        let b = others[2].0 + others[3].0;
        PublicKey((a + b).affine())
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.compress()
    }

    pub fn from_bytes(bytes: &[u8]) -> CtOption<PublicKey> {
        Point::decompress(bytes.try_into().unwrap()).map(PublicKey)
    }
}