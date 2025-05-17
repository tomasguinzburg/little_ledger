use std::collections::HashMap;

use super::{
    balance::Balance,
    common::{Client, Tx},
    transaction::{Deposit, Transaction, Type},
};
use anyhow::{Result, anyhow, bail};

#[derive(Debug)]
pub struct Account {
    pub client: Client,
    pub balance: Balance,
    pub locked: bool,
    pub deposits: HashMap<Tx, Deposit>,
}

impl Account {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            balance: Balance::default(),
            locked: false,
            deposits: HashMap::new(),
        }
    }

    pub fn process(&mut self, transaction: Transaction) -> Result<()> {
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

                let result = self.balance.hold(amount);
                if result.is_err() {
                    self.locked = true;
                }

                result
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
                self.locked = true;
                let deposit = self.get_deposit(transaction.tx)?;
                let amount = deposit.amount;

                deposit
                    .close_dispute()
                    .map_err(|e| anyhow!("{e} for {:?}", transaction.tx))?;

                self.balance.revert(amount)
            }
        }
    }

    fn get_deposit(&mut self, tx: Tx) -> Result<&mut Deposit> {
        self.deposits
            .get_mut(&tx)
            .ok_or(anyhow!("no deposit for resolve tx: {:?}", tx))
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
