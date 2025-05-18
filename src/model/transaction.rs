use anyhow::{Result, bail};

use super::common::{Amount, Client, Tx};

/// A transaction
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Transaction {
    /// The type of the transaction
    pub t_type: Type,
    /// The client for this transaction
    pub client: Client,
    /// The transaction id
    pub tx: Tx,
}

/// The type of a `Transaction`.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    /// A deposit
    Deposit(Deposit),
    /// A withdrawal
    Withdrawal(Withdrawal),
    /// A dispute
    Dispute,
    /// A resolve
    Resolve,
    /// A chargeback
    Chargeback,
}

/// Parameters for a deposit
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Deposit {
    /// The amount to deposit
    pub amount: Amount,
    /// A dispute status
    pub dispute_status: DisputeStatus,
}

/// Parameters for a withdrawal
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Withdrawal {
    /// The amount to withdraw
    pub amount: Amount,
}

/// The dispute status of a deposit
///
/// `Closed` the default status, no dispute is currently pending resolution
/// `Opened` a dispute is pending resolution
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum DisputeStatus {
    #[default]
    Closed,
    Opened,
}

impl Deposit {
    /// Opens a dispute on this deposit
    ///
    /// # Errors
    ///
    /// Returns `anyhow::Error` if a dispute is already opened
    pub fn open_dispute(&mut self) -> Result<()> {
        if let DisputeStatus::Opened = self.dispute_status {
            bail!("can't open a new dispute until the previous one is finalized");
        }
        self.dispute_status = DisputeStatus::Opened;
        Ok(())
    }

    /// Closes a dispute on this deposit
    ///
    /// # Errors
    ///
    /// Returns `anyhow::Error` unless there is an existing opened dispute
    pub fn close_dispute(&mut self) -> Result<()> {
        if let DisputeStatus::Closed = self.dispute_status {
            bail!("can't close unless there is an existing dispute");
        }
        self.dispute_status = DisputeStatus::Closed;
        Ok(())
    }
}
