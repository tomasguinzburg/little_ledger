# Payments Engine

A small payments engine that supports deposits, withdrawals, and some primitive
management of disputes from a CSV, and outputs the state of the transaction
ledger.

## Usage

As long as you have `cargo` installed, you should be able to just clone the repo
and then

`cargo run -- input.csv > output.csv`

The program is very lenient with what it's willing to parse, but if you want it
to report errors you can throw in a `--verbose` flag.

## Assumptions

### 1. Customers dispute transactions against a third party, i.e. cc issuer or bank

- `Deposits` are understood as top-ups from a third party account into our
  application's account.
- `Withdrawals` are understood as pay-outs from our application's account into a
  third party account.

Only deposits can be disputed by a customer (by making a claim to their credit
card issuer for a stolen card, for example), as there would be no point in
disputing a withdrawal: They could just go and deposit it again or return the
transfer themselves.

Disputing a withdrawal makes no point, why would they dispute against a third
party for money received? We could handle this case by allowing negative amounts
(either held or available) but that's a lot of added complexity for a long shot.

### 2. A resolved dispute cannot be chargedback nor viceversa

While there are some use cases where it might actually make sense to chargeback
a resolved dispute, in this engine we don't allowit unless a new dispute is
opened.

### 3. Opening a new dispute for the same tx before the previous one was resolved is a no-op

You can only have one open dispute per tx at the same time. You can always
reopen it **after** resolution, but not before.

### 4. Disputing funds that have already been withdrawn results in a locked acount

Because transactions appear in chronological ordering, it's possible to have an
account that receives:

- A deposit for any positive amount (i.e, 10.00 to available)
- A withdrawal for any positive amount (i.e, 10.00 from available)
- A dispute on the first deposit (10.00 from available to held, but there's
  nothing available)

In this case, there would be insufficient available funds to withold, and we
assume that's reason enough for the account to be locked immediately. There are
other ways to handle this but they increase complexity considerably.
