use super::common::Amount;
use anyhow::{Result, bail};

#[derive(Debug)]
pub struct Balance {
    pub available: Amount,
    pub held: Amount,
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            available: Amount(0.into()),
            held: Amount(0.into()),
        }
    }
}

impl Balance {
    pub fn total(&self) -> Amount {
        self.available + self.held
    }

    pub fn credit(&mut self, amount: Amount) {
        self.available += amount;
    }

    pub fn debit(&mut self, amount: Amount) -> Result<()> {
        if self.available >= amount {
            self.available -= amount;
            Ok(())
        } else {
            bail!("insufficient funds")
        }
    }
    //TODO: hold, release, revert
}
