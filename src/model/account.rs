use std::collections::HashMap;

use super::{
    balance::Balance,
    common::{Client, Tx},
    transaction::{Deposit, Transaction, Type},
};
use anyhow::{Result, anyhow, bail};

/// A client's account.
#[derive(Debug)]
pub struct Account {
    /// The client who owns the account.
    pub client: Client,
    /// the balance on this account.
    pub balance: Balance,
    /// This account's locked status.
    pub locked: bool,
    /// A set of deposits (as a `HashMap` for fast random access using the `Tx`)
    pub deposits: HashMap<Tx, Deposit>,
}

impl Account {
    // Initialize an account
    //
    // Creates a new account for the selected client with default balance, no deposits, and
    // unlocked.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            balance: Balance::default(),
            locked: false,
            deposits: HashMap::new(),
        }
    }

    /// Lock the account
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Apply a transaction
    ///
    /// Apply `transaction` to this account. If the transaction is compatible with the account's
    /// current state, it will process it, mutating it's status accordingly. Otherwise, it will
    /// return an error.
    ///
    /// # Errors
    ///
    /// Returns `anyhow::Error` whenever a transaction can be processed due to being
    /// inconsistent with the current account status.
    ///
    /// `apply` will always fail if the account is locked.
    ///
    /// `apply` will always fail for transactions belonging to a different client.
    pub fn apply(&mut self, transaction: Transaction) -> Result<()> {
        Account::bail_if_unauthorized(self, transaction.client)?;
        Account::bail_if_locked(self)?;

        match transaction.t_type {
            Type::Deposit(deposit) => {
                self.balance.credit(deposit.amount);
                self.deposits.insert(transaction.tx, deposit);
                Ok(())
            }
            Type::Withdrawal(withdrawal) => self
                .balance
                .debit(withdrawal.amount)
                .map_err(|e| anyhow!("{e} for tx {:?}", transaction.tx)),
            Type::Dispute => {
                let deposit = self.get_deposit(transaction.tx)?;
                let amount = deposit.amount;

                deposit
                    .open_dispute()
                    .map_err(|e| anyhow!("{e} for {:?}", transaction.tx))?;

                self.balance.hold(amount)

                // FIXME: I think we should lock the account here if hold().is_err(); as this looks like a
                // typical case of fraud, but I don't want to break the spec without discussing it
                // with whomever is going to consume this.
            }
            Type::Resolve => {
                let deposit = self.get_deposit(transaction.tx)?;
                let amount = deposit.amount;

                deposit
                    .close_dispute()
                    .map_err(|e| anyhow!("{e} for {:?}", transaction.tx))?;

                self.balance.release(amount)
            }
            Type::Chargeback => {
                self.lock();
                let deposit = self.get_deposit(transaction.tx)?;
                let amount = deposit.amount;

                deposit
                    .close_dispute()
                    .map_err(|e| anyhow!("{e} for {:?}", transaction.tx))?;

                self.balance.reimburse(amount)
            }
        }
    }

    fn get_deposit(&mut self, tx: Tx) -> Result<&mut Deposit> {
        self.deposits
            .get_mut(&tx)
            .ok_or(anyhow!("deposit missing tx: {:?}", tx))
    }

    fn bail_if_locked(&self) -> Result<()> {
        if self.locked {
            bail!("account is locked {}", self.client.0)
        }
        Ok(())
    }

    fn bail_if_unauthorized(&self, client: Client) -> Result<()> {
        if self.client != client {
            bail!("unauthorized")
        }
        Ok(())
    }
}
