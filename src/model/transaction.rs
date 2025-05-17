use serde::Serialize;

use super::common::{Amount, Client, Tx};

#[derive(Debug, PartialEq, Serialize)]
pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Deposit {
    pub client: Client,
    pub tx: Tx,
    pub amount: Amount,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Withdrawal {
    pub client: Client,
    pub tx: Tx,
    pub amount: Amount,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Dispute {
    pub client: Client,
    pub tx: Tx,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Resolve {
    pub client: Client,
    pub tx: Tx,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Chargeback {
    pub client: Client,
    pub tx: Tx,
}
