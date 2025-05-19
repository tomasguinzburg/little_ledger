use std::{error::Error, path::PathBuf};

use clap::{Parser, command};
use little_ledger::{
    io::{
        input::{create_csv_reader, deserialize_transactions},
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
/// process the valid lines. It will only return with err if the specified file can't be opened (i.e. does not exist),
/// or if a buffer to stdout can't be flushed, which should never happen.
fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line options.
    let cli = Cli::parse();
    if cli.verbose {
        eprintln!("Verbose mode enabled, printing all errors to stderr.");
    }
    let rdr = create_csv_reader(cli.input_path)?;

    let txns = deserialize_transactions(Some(rdr), cli.verbose)?;

    let ledger = process_transactions(txns, cli.verbose);

    serialize_ledger(ledger, None, cli.verbose)?;

    Ok(())
}

/// Process transactions
///
/// Create a new ledger and apply all `txns` in the provider iterator. Returns the fully processed
/// ledger.
pub fn process_transactions(txns: impl Iterator<Item = Transaction>, verbose: bool) -> Ledger {
    let mut ledger = Ledger::default();

    // Then process all transactions.
    txns.for_each(|txn| {
        if let Err(e) = ledger.apply(txn) {
            if verbose {
                eprintln!("warning: {e}");
            }
        }
    });

    ledger
}
