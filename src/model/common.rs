use std::ops::{AddAssign, SubAssign};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct Client(pub u16);

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Tx(pub u32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Serialize, Deserialize)]
pub struct Amount(#[serde(with = "rust_decimal::serde::arbitrary_precision")] pub Decimal);

impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
