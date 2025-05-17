use super::{
    balance::Balance,
    common::Client,
    transaction::{Deposit, Transaction, Withdrawal},
};
use anyhow::{Result, bail};

#[derive(Debug)]
pub struct Account {
    pub client: Client,
    pub balance: Balance,
    pub locked: bool,
    pub history: Vec<Transaction>,
}

impl Account {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            balance: Balance::default(),
            locked: false,
            history: Vec::new(),
        }
    }

    pub fn deposit(&mut self, tx: Deposit) -> Result<()> {
        if self.locked {
            bail!("account is locked {}", self.client.0) //TODO: impl Display
        }
        self.balance.credit(tx.amount);
        self.history.push(Transaction::Deposit(tx));
        Ok(())
    }

    pub fn withdraw(&mut self, tx: Withdrawal) -> Result<()> {
        if self.locked {
            bail!("account is locked {}", self.client.0) //TODO: impl Display
        }
        self.balance.debit(tx.amount)?;
        self.history.push(Transaction::Withdrawal(tx));
        Ok(())
    }
}
