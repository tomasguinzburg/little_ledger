use std::io::{BufWriter, Write, stdout};

use csv::Writer;
use serde::Serialize;

use crate::model::{
    account::Account,
    common::{Amount, Client},
    ledger::Ledger,
};

#[derive(Debug, Serialize)]
pub struct OutputAccountRecord {
    client: Client,
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

impl From<Account> for OutputAccountRecord {
    fn from(acc: Account) -> Self {
        OutputAccountRecord {
            client: acc.client,
            available: acc.balance.available(),
            held: acc.balance.held(),
            total: acc.balance.total(),
            locked: acc.locked,
        }
    }
}

pub fn writer<W: Write>(w: W) -> Writer<W> {
    csv::WriterBuilder::new()
        .has_headers(true)
        .flexible(false)
        .from_writer(w)
}

pub fn serialize_ledger(ledger: Ledger, verbose: bool) {
    let mut csv_writer = writer(BufWriter::new(stdout()));

    for acc in ledger.accounts.into_values() {
        if let Err(e) = csv_writer.serialize(OutputAccountRecord::from(acc)) {
            if verbose {
                eprintln!("serialization_error: {e}");
            }
        }
    }
}
