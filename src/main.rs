use std::{error::Error, fs::File, io::BufReader};

use io::input::{self, InputMappingError, InputTransactionRecord};
use model::{ledger::Ledger, transaction::Transaction};

mod io;
mod model;

const PATH: &str = "./transactions.csv";

/// This program is prepared to handle input pesimistically:
///     It assumes only stdout matters, and stderr can be trated as a log,
///     which seems valid for these kind of requirements `cargo run -- input.csv > output.csv`.
///     Ideally, when dealing with transactions, users of this program should ensure their
///     input CSVs are not malformed.
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
                // Should be <WARN if using a logger, as this is expected.
                eprintln!("ledger_processing_error: {e}");
            }
        });

    // TODO: output ledger to stdout as CSV
    // ledger.print();

    Ok(())
}
