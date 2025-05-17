use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, stdout},
};

use io::{
    input::{self, InputMappingError, InputTransactionRecord},
    output::{self, OutputAccountRecord},
};
use model::{ledger::Ledger, transaction::Transaction};

mod io;
mod model;

const PATH: &str = "./transactions.csv";

/// This program is prepared to handle more or less malformed input:
///     It assumes only stdout matters, and stderr can be trated as a log,
///     which seems valid for these kind of requirements `cargo run -- input.csv > output.csv`.
fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open(PATH)?;
    let mut csv_reader = input::reader(BufReader::new(file));

    // Read all valid records, dump parse errors to stderr.
    let records = csv_reader
        .deserialize::<InputTransactionRecord>()
        .filter_map(|r_itr| match r_itr {
            Ok(itr) => Some(itr),
            Err(e) => {
                eprintln!(
                    "input_mapping_error::parse_error: {}",
                    InputMappingError::ParseError(e)
                );
                None
            }
        });

    let mut ledger = Ledger::default();

    // Attempt to map all `InputTransactionRecord`s into `Transaction`s.
    records
        .map(Transaction::try_from)
        .filter_map(|rtxn| match rtxn {
            Ok(txn) => Some(ledger.process(txn)),
            Err(e) => {
                eprintln!("input_mapping_error: {e}");
                None
            }
        })
        // Then process all transactions.
        .for_each(|processing_result| {
            if let Err(e) = processing_result {
                eprintln!("warning: {e}");
            }
        });

    // Serialize ledger
    let mut csv_writer = output::writer(BufWriter::new(stdout()));
    for acc in ledger.accounts.into_values() {
        csv_writer
            .serialize(OutputAccountRecord::from(acc))
            .expect("Should be serializable no errors"); //TODO: this is dangerous
    }

    Ok(())
}
