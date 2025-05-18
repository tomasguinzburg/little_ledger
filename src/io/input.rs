use std::{
    io::{BufReader, Read, stdin},
    path::PathBuf,
};

use csv::Reader;
use serde::Deserialize;
use thiserror::Error;

use crate::model::{
    common::{Amount, Client, Tx},
    transaction::{Deposit, DisputeStatus, Transaction, Type, Withdrawal},
};

pub fn csv_reader<R: Read>(rdr: R) -> Reader<R> {
    csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(true)
        .flexible(true)
        .from_reader(rdr)
}

pub fn create_csv_reader(
    input_path: Option<PathBuf>,
) -> anyhow::Result<csv::Reader<Box<dyn Read>>> {
    let buf_reader: Box<dyn Read> = match input_path {
        Some(path) => {
            let file = std::fs::File::open(path)?;
            Box::new(BufReader::new(file))
        }
        None => Box::new(BufReader::new(stdin())),
    };

    Ok(csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(true)
        .flexible(true)
        .from_reader(buf_reader))
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
    #[error("missing mandatory amount for a {transaction_type:?} - {tx:?}")]
    MissingAmount {
        transaction_type: TransactionType,
        tx: Tx,
    },
    #[error("line {0} could not be parsed")]
    ParseError(#[from] csv::Error),
}

impl TryFrom<InputTransactionRecord> for Transaction {
    type Error = InputMappingError;

    fn try_from(raw_record: InputTransactionRecord) -> Result<Self, Self::Error> {
        let tx = raw_record.tx;
        let client = raw_record.client;
        let transaction_type = raw_record.transaction_type;

        match raw_record.transaction_type {
            TransactionType::Deposit => Ok(Transaction {
                client,
                tx,
                t_type: Type::Deposit(Deposit {
                    dispute_status: DisputeStatus::default(),
                    amount: raw_record.amount.ok_or(InputMappingError::MissingAmount {
                        transaction_type,
                        tx,
                    })?,
                }),
            }),
            TransactionType::Withdrawal => Ok(Transaction {
                client,
                tx,
                t_type: Type::Withdrawal(Withdrawal {
                    amount: raw_record.amount.ok_or(InputMappingError::MissingAmount {
                        transaction_type,
                        tx,
                    })?,
                }),
            }),
            TransactionType::Dispute => Ok(Transaction {
                t_type: Type::Dispute,
                client,
                tx,
            }),
            TransactionType::Resolve => Ok(Transaction {
                t_type: Type::Resolve,
                client,
                tx,
            }),
            TransactionType::Chargeback => Ok(Transaction {
                t_type: Type::Chargeback,
                client,
                tx,
            }),
        }
    }
}
