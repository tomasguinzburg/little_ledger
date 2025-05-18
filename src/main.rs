use std::{error::Error, io::Read, path::PathBuf};

use clap::{Parser, command};
use payments::{
    io::{
        input::{InputMappingError, InputTransactionRecord, create_csv_reader},
        output::serialize_ledger,
    },
    model::{ledger::Ledger, transaction::Transaction},
};

/// Command line arguments for the Petit Payments Engine.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Cli {
    /// Optional path to the input CSV file.
    ///
    /// The application will read from stdin if not provided.
    #[arg(index = 1)]
    input_path: Option<PathBuf>,

    /// Enables verbose output.
    ///
    /// When set (-v or --verbose), the application will print errors and warnings to stderr,
    /// otherwise it will ignore them silently.
    #[arg(short, long)]
    verbose: bool,
}

/// Petit Payments Engine (PPE).
///
/// Reads transactions from an input csv, tallies them on a ledger, and outputs the state of the
/// ledger as a csv.
///
/// # Errors
///
/// The application will try to continue optimistically as best as it can, even on a malformed CSV it will try to
/// process the valid lines. It will only return with err if the specified file can't be opened (i.e. does not exist)
fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line options.
    let cli = Cli::parse();
    if cli.verbose {
        eprintln!("Verbose mode enabled, printing all errors to stderr.");
    }

    let mut rdr = create_csv_reader(cli.input_path)?;

    let ledger = process_transactions(&mut rdr, cli.verbose);

    serialize_ledger(ledger, cli.verbose);

    Ok(())
}

/// Process transactions from a csv reader
pub fn process_transactions(rdr: &mut csv::Reader<Box<dyn Read>>, verbose: bool) -> Ledger {
    let mut ledger = Ledger::default();

    // Read all valid records, dump parse errors to stderr.
    let rcrds =
        rdr.deserialize::<InputTransactionRecord>()
            .filter_map(|txn_record| match txn_record {
                Ok(itr) => Some(itr),
                Err(e) => {
                    let e = InputMappingError::ParseError(e);
                    if verbose {
                        eprintln!("input_mapping_error::parse_error: {e}",);
                    }
                    None
                }
            });

    // Attempt to map all `InputTransactionRecord`s into `Transaction`s.
    let txns = rcrds
        .map(Transaction::try_from)
        .filter_map(|txn| match txn {
            Ok(txn) => Some(ledger.apply(txn)),
            Err(e) => {
                if verbose {
                    eprintln!("input_mapping_error: {e}");
                }
                None
            }
        });

    // Then process all transactions.
    txns.for_each(|processing_result| {
        if let Err(e) = processing_result {
            if verbose {
                eprintln!("warning: {e}");
            }
        }
    });

    ledger
}
