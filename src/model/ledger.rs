use std::collections::HashMap;

use anyhow::Result;

use super::{account::Account, common::Client, transaction::Transaction};

/// A ledger
///
/// A ledger represents the status of a set of accounts after applying a set of transactions to
/// them.
#[derive(Debug, Default)]
pub struct Ledger {
    /// The set of accounts (as a `HashMap` for fast random access using the `Client`)
    pub accounts: HashMap<Client, Account>,
}

impl Ledger {
    /// Apply a transaction
    ///
    /// Apply a `transaction` to this ledger. The ledger is responsible of routing it to the
    /// correct account.
    ///
    /// # Errors
    /// Returns `anyhow::Error` if the transaction fails to be processed.
    pub fn apply(&mut self, txn: Transaction) -> Result<()> {
        self.get_account_for(txn.client).apply(txn)
    }

    /// Get the account for a client
    ///
    /// Returns the existing account , or a new one if it doesn't exist.
    pub fn get_account_for(&mut self, client: Client) -> &mut Account {
        self.accounts
            .entry(client)
            .or_insert_with(|| Account::new(client))
    }
}
