use std::ops::{Add, AddAssign, Sub, SubAssign};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The client id
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct Client(pub u16);

/// The transaction id
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct Tx(pub u32);

/// An ad-hoc type that represents non-negative, arbitrary precision unitless amounts.
///
/// Besides derivations, it supports add, sub, addassign and subassign. It can be constructed by
/// `try_from` a decimal.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "rust_decimal::Decimal")]
pub struct Amount(#[serde(with = "rust_decimal::serde::arbitrary_precision")] Decimal);

#[derive(Error, Debug)]
#[error("negative amounts are not allowed")]
pub struct NegativeAmountError;

impl TryFrom<Decimal> for Amount {
    type Error = NegativeAmountError;

    /// Create from a `Decimal`, as long as the decimal is positive.
    fn try_from(d: Decimal) -> Result<Self, Self::Error> {
        if d < Decimal::ZERO {
            Err(NegativeAmountError)
        } else {
            Ok(Amount(d))
        }
    }
}

/// Add two amounts together
impl Add for Amount {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Amount(self.0 + rhs.0)
    }
}

/// Subtract rhs from lhs
///
/// Clips at 0
impl Sub for Amount {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.0 < rhs.0 {
            Amount::ZERO
        } else {
            Amount(self.0 - rhs.0)
        }
    }
}

/// Add two amounts and store the result on lhs
impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

/// Sub rhs from lhs and store the result on lhs
///
/// Clips at 0
impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        if self.0 < rhs.0 {
            self.0 = Amount::ZERO.0;
        } else {
            self.0 -= rhs.0;
        }
    }
}

/// Expose ZERO as constructor.
///
/// We could have implemented `Default`, but using a constant ZERO is much more explicit about which is the
/// default value.
impl Amount {
    pub const ZERO: Amount = Amount(Decimal::ZERO);
}

#[cfg(test)]
//We provide a few amounts for testing ergonomy
impl Amount {
    pub const ONE: Amount = Amount(Decimal::ONE);
    pub const TWO: Amount = Amount(Decimal::TWO);
    pub const TEN: Amount = Amount(Decimal::TEN);
}
