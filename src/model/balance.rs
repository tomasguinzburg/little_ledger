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

    pub fn hold(&mut self, amount: Amount) -> Result<()> {
        if self.available >= amount {
            self.available -= amount;
            self.held += amount;
            Ok(())
        } else {
            bail!("insufficient funds") // They have already escaped with the money :(
        }
    }

    pub fn release(&mut self, amount: Amount) -> Result<()> {
        if self.held >= amount {
            self.held -= amount;
            self.available += amount;
            Ok(())
        } else {
            bail!("insufficient funds on hold") // Should never happen within this program
        }
    }

    pub fn revert(&mut self, amount: Amount) -> Result<()> {
        if self.held >= amount {
            self.held -= amount;
            Ok(())
        } else {
            bail!("insufficient funds on hold") // Should never happen within this program.
        }
    }
}
