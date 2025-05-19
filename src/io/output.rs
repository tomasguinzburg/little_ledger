use std::{
    io::{BufWriter, Write, stdout},
    path::PathBuf,
};

use anyhow::{Context, Result};
use csv::Writer;
use serde::Serialize;

use crate::model::{
    account::Account,
    common::{Amount, Client},
    ledger::Ledger,
};

/// Serialize a ledger to a target writable, or stdout
///
/// # Errors
///
/// Returns `anyhow::Error` if it fails to flush the buffer.
pub fn serialize_ledger<'a>(
    ledger: Ledger,
    wtr: Option<csv::Writer<Box<dyn Write + 'a>>>,
    verbose: bool,
) -> Result<()> {
    let mut csv_writer: csv::Writer<Box<dyn Write + 'a>> = match wtr {
        Some(w) => w,
        None => writer(Box::new(stdout())),
    };

    for acc in ledger.accounts.into_values() {
        if let Err(e) = csv_writer.serialize(OutputAccountRecord::from(acc)) {
            if verbose {
                eprintln!("serialization_error: {e}");
            }
        }
    }

    csv_writer
        .flush()
        .with_context(|| "failed to flush the buffer")
}

/// Create a file CSV writer
///
/// Will write to file or stdout
///
/// # Errors
///
/// Returns `anyhow::Error` if the file can not be opened.
pub fn create_csv_writer(
    output_path: Option<PathBuf>,
) -> anyhow::Result<csv::Writer<Box<dyn Write>>> {
    let buf_writer: Box<dyn Write> = match output_path {
        Some(path) => {
            let file = std::fs::File::create(path)?;
            Box::new(BufWriter::new(file))
        }
        None => Box::new(BufWriter::new(stdout())),
    };

    Ok(writer(buf_writer))
}

pub fn writer<W: Write>(w: W) -> Writer<W> {
    csv::WriterBuilder::new()
        .has_headers(true)
        .flexible(false)
        .from_writer(w)
}

#[derive(Debug, Serialize)]
struct OutputAccountRecord {
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
