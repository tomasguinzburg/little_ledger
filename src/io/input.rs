use std::io::Read;

use csv::Reader;
use serde::Deserialize;
use thiserror::Error;

use crate::model::{
    common::{Amount, Client, Tx},
    transaction::{Deposit, Transaction, Withdrawal},
};

pub fn reader<R: Read>(rdr: R) -> Reader<R> {
    csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(true)
        .flexible(true)
        .from_reader(rdr)
}

#[derive(Debug, Deserialize)]
pub struct InputTransactionRecord {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: Client,
    tx: Tx,
    amount: Option<Amount>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Error, Debug)]
pub enum InputMappingError {
    #[error("unknown transaction type {0}")]
    UnknownType(String),
    #[error("missing mandatory amount for a {transaction_type:?} - {tx:?} - {client:?}")]
    MissingAmount {
        transaction_type: TransactionType,
        tx: Tx,
        client: Client,
    },
    #[error("unexpected amount for a {transaction_type:?} - {tx:?} - {client:?}")]
    UnexpectedAmount {
        transaction_type: TransactionType,
        tx: Tx,
        client: Client,
    },
    #[error("line {0} could not be parsed")]
    ParseError(#[from] csv::Error),
}

impl TryFrom<InputTransactionRecord> for Transaction {
    type Error = InputMappingError;

    fn try_from(raw: InputTransactionRecord) -> Result<Self, Self::Error> {
        let tx = raw.tx;
        let client = raw.client;
        let transaction_type = raw.transaction_type;

        match raw.transaction_type {
            TransactionType::Deposit => {
                let amount = raw.amount.ok_or(InputMappingError::MissingAmount {
                    transaction_type,
                    tx,
                    client,
                })?;
                Ok(Transaction::Deposit(Deposit { client, tx, amount }))
            }
            TransactionType::Withdrawal => {
                let amount = raw.amount.ok_or(InputMappingError::MissingAmount {
                    transaction_type,
                    tx,
                    client,
                })?;
                Ok(Transaction::Withdrawal(Withdrawal { client, tx, amount }))
            }
            _ => Err(InputMappingError::UnknownType(
                "not implemented".to_string(),
            )),
        }
    }
}
