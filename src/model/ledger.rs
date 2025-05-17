use std::collections::HashMap;

use anyhow::Result;

use super::{account::Account, common::Client, transaction::Transaction};

#[derive(Debug, Default)]
pub struct Ledger {
    pub accounts: HashMap<Client, Account>,
}

impl Ledger {
    pub fn process(&mut self, txn: Transaction) -> Result<()> {
        self.account_for(txn.client).process(txn)
    }

    pub fn account_for(&mut self, client: Client) -> &mut Account {
        self.accounts
            .entry(client)
            .or_insert_with(|| Account::new(client))
    }
}
