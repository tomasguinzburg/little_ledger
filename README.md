# Petit Payments Engine (PPE)

A small payments engine that reads transactions from an input csv, tallies them
on a ledger, and outputs the state of the ledger as a csv. It has some minimal
dispute management functionality.

## Usage

With `cargo` installed, the repository can be cloned and then executed via:

```sh
cargo run -- input.csv > output.csv
```

The program exhibits leniency in parsing, ignoring any error and processing al
syntactically and semantically valid lines on the input. Error reporting to
stderr can be enabled by using the `--verbose` flag:

```sh
cargo run -- input.csv --verbose > output.csv
```

You can also pipe stdin, for example:

```sh
cat input.csv | cargo run
```

A dev shell is provided in `flake.nix` if you want to run reproductibly.

## Assumptions

### 1. Customers dispute transactions against a third party, i.e. cc issuer or bank

- `Deposits` are understood as top-ups from a third party account into the
  application's account.
- `Withdrawals` are understood as pay-outs from the application's account into a
  third party account.

Only deposits can be disputed by a customer (by making a claim to their credit
card issuer for a stolen card, for example), as disputing a withdrawal offers no
clear advantage; the customer could deposit the funds again or return the
transfer.

Disputing a withdrawal is considered out of scope, as the rationale for a
customer disputing received funds against a third party is unclear. Handling
such a case by allowing negative amounts (either held or available) would
introduce significant complexity for a marginal use case.

### 2. A resolved dispute cannot be chargedback nor viceversa

While some use cases might suggest allowing a chargeback on a resolved dispute,
this engine does not permit such an action unless a new dispute is initiated.
Chargebacks and resolves are only processed if an associated deposit exists and
is currently in an open dispute state.

### 3. Opening a new dispute for the same tx before the previous one was resolved is a no-op

Only one open dispute per transaction is permitted at any given time. A dispute
can always be reopened **after** resolution, but not prior to it.

### 4. Disputing funds that have already been withdrawn results in a locked account

Because transactions appear in chronological ordering, it's possible to have an
account that receives a deposit, a withdrawal and a dispute, such that:

- Deposit any positive amount (i.e, 10.00 to available)
- Withdraw any positive amount (i.e, 10.00 from available)
- Dispute the first deposit (10.00 should be moved from available to held, but
  there's nothing available to move)

**In this case**, there would be insufficient available funds to withhold. This
seems like a standard case of fraud (disputing a deposit after withdrawing it's
funds) so **the account is locked**. Alternative handling methods exist but
would introduce considerable complexity.

## Design

The design of the program is minimal: a `model` module contains all of the core
models, while some serialization and deserialization concerns are grouped under
the `io` module.

Because transactions can be thought of as events, and the input csv as an append
only log, the state of the ledger can always be reconstructed. The initial
design considered keeping the whole history of transactions associated with each
account (even ignored ones) for auditing purposes.

However, for performance and simplicity in handling disputes, only a history of
deposits is saved. The original CSV remains available, along with the list of
processed records.

### Model

- `model/transaction.rs`: A **transaction**. An enum was initially considered
  for representing transactions, but most data was common to all of them, with
  only a few fields depending on the type.
  - `Type::Deposit`: a deposit of funds, results on a credit to the balance. Can
    be rolled back by disputing it.
  - `Type::Withdrawal`: a withdrawal, results on a debit to the balance. It is
    assumed it cannot be rolled back.
  - `Type::Dispute`: a dispute to any transaction, but will only work on
    deposits and be ignored otherwise. Results in withholding funds.
  - `Type::Resolve`: a positive dispute resolution. Works only on **already
    disputed deposits**, clears the dispute status, and results in releasing
    funds.
  - `Type::Chargeback`: a negative dispute resolution. Works only on **already
    disputed deposits**, clears the dispute status, and results in reverting
    funds.
- `model/account.rs`: A client's **account**. Has a **client** id, a
  **balance**, a list (as a hash for random access) of **deposits**, and a
  killswitch **lock**. Once an account is locked, all transactions against it
  fail, and there is no mechanism for unlocking it.
- `model/balance.rs`: A client's **balance**. Has an **available** amount, a
  **held** amount, and several utilities for safely performing operations on
  these fields.
- `model/ledger.rs`: A **ledger** is a representation of all accounts in the
  system after a certain number of transactions. A ledger can process arbitrary
  transactions by finding the account they affect, and delegating processing to
  it.
- `model/common.rs`: Common types that are ubiquitous to this domain. The
  `Amount` type might be an unnecessary abstraction; using `Decimal` directly
  could have been preferable, avoiding the need to reimplement arithmetic
  operations.

### IO

Both `io/input.rs` and `io/output.rs` implement a deserializable and
serializable intermediate representation, which serves the purpose of isolating
the models from the contract specifications.

Notably, `io/input.rs` implements
`TryFrom<InputTransactionRecord> for Transaction`, and `io/output.rs` implements
`From<Account> for OutputAccountRecord`.

## Dependencies

The number of crates might appear extensive for a project of this scope. The
selection criteria were that they addressed a specific issue, were common
enough, deemed generally secure by the Rust community, and at least >1.0.0.

- `serde` and `csv`: these are fundamental to the exercise.
- `rust_decimal`: The choice of `rust_decimal` might be considered conservative.
  Because there's no multiplication, exponentiation nor division, `f64` might
  have sufficed for the required precision; but using arbitrary precision
  decimals protects against platform indeterminism.
- `anyhow` and `thiserror`: facilitate easier error handling, although their
  integration could be further refined.
- `clap`: provides clutter-free arguments parsing. For easy testing and
  debugging, the aim was to include a verbose flag and also to be able to pipe
  stdin, without risking harm to the intended usage.

### Dev dependencies

- `rust_decimal_macros`: `dec!(3.141)` as syntax sugar for
  `Decimal::new(3141, 3)`. It adds up on the fingers.

## Improvements

- As the solution evolved, the intermediate representations and the models
  converged. Some slight changes in modelling and effective use of serde
  annotations might allow for the removal of some of these models if desirable.
- Several components are lazily evaluated, but in general explicit concurrency
  is limited. For some datasets, grouping first by client and then processing
  transactions in parallel might be a worthy tradeoff, but it could be
  detrimental for other datasets.
- Transaction ordering on the CSV has semantical meaning: they are in
  chronological order. However that information is completely ignored.
- In general, a significant performance improvement could be achieved by being
  able to print a part of the ledger as some accounts are still being processed.
