use super::common::Amount;
use anyhow::{Result, bail};

/// The balance of an account.
///
/// Balances can only be created by default, and their internal fields can only be modified through
/// their public API, which ensures validations apply.
#[derive(Debug)]
pub struct Balance {
    /// The available funds.
    available: Amount,
    /// The funds on hold.
    held: Amount,
}

impl Default for Balance {
    /// Default balance with available and held funds initalized to `Amount::ZERO`
    fn default() -> Self {
        Self {
            available: Amount::ZERO,
            held: Amount::ZERO,
        }
    }
}

impl Balance {
    /// The currently available funds
    #[must_use]
    pub fn available(&self) -> Amount {
        self.available
    }

    /// The funds currently on hold
    #[must_use]
    pub fn held(&self) -> Amount {
        self.held
    }

    /// The total funds
    ///
    /// Returns the total balance regardless of status, i.e. `available` + `held`.
    #[must_use]
    pub fn total(&self) -> Amount {
        self.available + self.held
    }

    /// Perform a credit
    ///
    /// Adds `amount` funds to the available balance.
    pub fn credit(&mut self, amount: Amount) {
        self.available += amount;
    }

    /// Perform a debit
    ///
    /// Subs `amount` funds from the available balance, if there's sufficient funds.
    ///
    /// # Errors
    /// `anyhow::Error` on insufficient funds.
    pub fn debit(&mut self, amount: Amount) -> Result<()> {
        if self.available >= amount {
            self.available -= amount;
            Ok(())
        } else {
            bail!("insufficient funds")
        }
    }

    /// Put funds on hold
    ///
    /// Moves `amount` funds from available to on hold, if there's sufficient funds.
    ///
    /// # Errors
    /// `anyhow::Error` on inssuficient funds.
    pub fn hold(&mut self, amount: Amount) -> Result<()> {
        if self.available >= amount {
            self.available -= amount;
            self.held += amount;
            Ok(())
        } else {
            bail!("insufficient funds")
        }
    }

    /// Release funds from hold
    ///
    /// Moves `amount` funds from `held` to `available`, if there's sufficient funds on hold.
    ///
    /// # Errors
    /// `anyhow::Error` on inssuficient funds on hold.
    pub fn release(&mut self, amount: Amount) -> Result<()> {
        if self.held >= amount {
            self.held -= amount;
            self.available += amount;
            Ok(())
        } else {
            bail!("insufficient funds on hold")
        }
    }

    /// Reimburse held funds
    ///
    /// Substracts `amount` funds from `held`, if there's sufficient funds on hold.
    ///
    /// # Errors
    /// `anyhow::Error` on inssuficient funds on hold.
    pub fn reimburse(&mut self, amount: Amount) -> Result<()> {
        if self.held >= amount {
            self.held -= amount;
            Ok(())
        } else {
            bail!("insufficient funds on hold")
        }
    }
}
