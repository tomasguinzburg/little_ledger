use anyhow::{Result, bail};

use super::common::{Amount, Client, DisputeStatus, Tx};

#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub t_type: Type,
    pub client: Client,
    pub tx: Tx,
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, PartialEq)]
pub struct Deposit {
    pub amount: Amount,
    pub dispute_status: DisputeStatus,
}

impl Deposit {
    pub fn open_dispute(&mut self) -> Result<()> {
        if let DisputeStatus::Opened = self.dispute_status {
            bail!("can't open a new dispute until the previous one is finalized");
        }
        self.dispute_status = DisputeStatus::Opened;
        Ok(())
    }

    pub fn close_dispute(&mut self) -> Result<()> {
        if let DisputeStatus::Closed = self.dispute_status {
            bail!("can't close unless there is an existing dispute");
        }
        self.dispute_status = DisputeStatus::Closed;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Withdrawal {
    pub amount: Amount,
}
