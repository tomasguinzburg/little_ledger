use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Read, stdin, stdout},
    path::PathBuf,
};

use clap::{Parser, command};
use io::{
    input::{self, InputMappingError, InputTransactionRecord},
    output::{self, OutputAccountRecord},
};
use model::{ledger::Ledger, transaction::Transaction};

mod io;
mod model;

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

    // Prepare the CSV reader with the input file or stdin if no path is passed.
    let buf_reader: Box<dyn Read> = match cli.input_path {
        Some(path) => {
            let file = File::open(path)?;
            Box::new(BufReader::new(file))
        }
        None => Box::new(BufReader::new(stdin())),
    };
    let mut csv_reader = input::reader(buf_reader);

    // Read all valid records, dump parse errors to stderr.
    let records = csv_reader
        .deserialize::<InputTransactionRecord>()
        .filter_map(|r_itr| match r_itr {
            Ok(itr) => Some(itr),
            Err(e) => {
                if cli.verbose {
                    eprintln!(
                        "input_mapping_error::parse_error: {}",
                        InputMappingError::ParseError(e)
                    );
                }
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
                if cli.verbose {
                    eprintln!("input_mapping_error: {e}");
                }
                None
            }
        })
        // Then process all transactions.
        .for_each(|processing_result| {
            if let Err(e) = processing_result {
                if cli.verbose {
                    eprintln!("warning: {e}");
                }
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
