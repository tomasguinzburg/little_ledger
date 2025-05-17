use std::collections::HashMap;

use anyhow::{Result, bail};

use super::{account::Account, common::Client, transaction::Transaction};

#[derive(Debug, Default)]
pub struct Ledger {
    pub accounts: HashMap<Client, Account>,
}

impl Ledger {
    pub fn process(&mut self, trx: Transaction) -> Result<()> {
        match trx {
            Transaction::Deposit(d) => self.account_for(d.client).deposit(d),
            Transaction::Withdrawal(w) => self.account_for(w.client).withdraw(w),
            _ => bail!("not implemented"),
        }
    }

    pub fn account_for(&mut self, client: Client) -> &mut Account {
        self.accounts
            .entry(client)
            .or_insert_with(|| Account::new(client))
    }
}
