use std::{
    io::{BufReader, Read, stdin},
    path::PathBuf,
};

use csv::Reader;
use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

use crate::model::{
    common::{Amount, Client, Tx},
    transaction::{Deposit, DisputeStatus, Transaction, Type, Withdrawal},
};

/// Creates a file CSV reader
///
/// Reads from the `input_path` or stdin if `input_path` is `None`.
///
/// # Errors
///
/// Returns `anyhow::Error` if the file can not be opened.
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

    Ok(reader(buf_reader))
}

/// Deserialize transactions
///
/// Deserializes all transactions from a given CSV reader
///
/// # Errors
///
/// Returns `anyhow::Error` if the file at `input_path` can't be opened.
pub fn deserialize_transactions(
    rdr: Option<csv::Reader<Box<dyn Read>>>,
    verbose: bool,
) -> anyhow::Result<impl Iterator<Item = Transaction>> {
    let rdr: csv::Reader<Box<dyn Read>> = match rdr {
        Some(r) => r,
        None => reader(Box::new(BufReader::new(stdin()))),
    };

    Ok(map_transactions(deserialize_records(rdr, verbose), verbose))
}

/// Generic csv reader for anything that can be `Read`
pub fn reader<R: Read>(rdr: R) -> Reader<R> {
    csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(true)
        .flexible(true)
        .from_reader(rdr)
}

fn deserialize_records(
    rdr: csv::Reader<Box<dyn Read>>,
    verbose: bool,
) -> impl Iterator<Item = InputTransactionRecord> {
    rdr.into_deserialize::<InputTransactionRecord>().filter_map(
        move |txn_record| match txn_record {
            Ok(itr) => Some(itr),
            Err(e) => {
                let e = InputMappingError::ParseError(e);
                if verbose {
                    eprintln!("input_mapping_error::parse_error: {e}",);
                }
                None
            }
        },
    )
}

fn map_transactions(
    records: impl Iterator<Item = InputTransactionRecord>,
    verbose: bool,
) -> impl Iterator<Item = Transaction> {
    records
        .map(Transaction::try_from)
        .filter_map(move |txn| match txn {
            Ok(txn) => Some(txn),
            Err(e) => {
                if verbose {
                    eprintln!("input_mapping_error: {e}");
                }
                None
            }
        })
}

#[derive(Debug, Deserialize)]
pub struct InputTransactionRecord {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: Client,
    tx: Tx,
    #[serde(with = "rust_decimal::serde::str_option")]
    amount: Option<Decimal>,
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
    #[error("invalid amount {dec_amount} for {tx:?}")]
    InvalidAmount { dec_amount: Decimal, tx: Tx },
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
            TransactionType::Deposit => {
                let dec_amount = raw_record.amount.ok_or(InputMappingError::MissingAmount {
                    transaction_type,
                    tx,
                })?;
                let amount = Amount::try_from(dec_amount)
                    .or(Err(InputMappingError::InvalidAmount { tx, dec_amount }))?;
                Ok(Transaction {
                    client,
                    tx,
                    t_type: Type::Deposit(Deposit {
                        dispute_status: DisputeStatus::default(),
                        amount,
                    }),
                })
            }
            TransactionType::Withdrawal => {
                let dec_amount = raw_record.amount.ok_or(InputMappingError::MissingAmount {
                    transaction_type,
                    tx,
                })?;
                let amount = Amount::try_from(dec_amount)
                    .or(Err(InputMappingError::InvalidAmount { tx, dec_amount }))?;

                Ok(Transaction {
                    client,
                    tx,
                    t_type: Type::Withdrawal(Withdrawal { amount }),
                })
            }
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
