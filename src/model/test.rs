use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::{
    common::{Amount, Client, Tx},
    ledger::Ledger,
    transaction::{Deposit, Transaction, Withdrawal},
};

#[test]
fn test_withdrawal_creation() {
    let txn = Transaction::Withdrawal(Withdrawal {
        client: Client(1),
        tx: Tx(1),
        amount: Amount(dec!(3.14)),
    });

    if let Transaction::Withdrawal(wrd) = txn {
        assert_eq!(wrd.client, Client(1));
    }
}

#[test]
fn test_ledger() {
    let mut ledger = Ledger::default();
    let trxs = [
        Transaction::Deposit(Deposit {
            client: Client(1),
            tx: Tx(1),
            amount: Amount(dec!(10.00)),
        }),
        Transaction::Deposit(Deposit {
            client: Client(1),
            tx: Tx(2),
            amount: Amount(dec!(10.00)),
        }),
        Transaction::Withdrawal(Withdrawal {
            client: Client(1),
            tx: Tx(3),
            amount: Amount(dec!(5.00)),
        }),
    ];
    trxs.map(|trx| ledger.process(trx).expect("happy path shouldn't err"));

    assert_eq!(
        ledger.account_for(Client(1)).balance.available,
        Amount(dec!(15.00))
    );
}

#[test]
fn test_ledger_errs_on_insufficient_funds() {
    let mut ledger = Ledger::default();
    let trx_0 = Transaction::Deposit(Deposit {
        client: Client(1),
        tx: Tx(1),
        amount: Amount(dec!(10.00)),
    });
    let trx_1 = Transaction::Deposit(Deposit {
        client: Client(1),
        tx: Tx(2),
        amount: Amount(dec!(10.00)),
    });
    let trx_2 = Transaction::Withdrawal(Withdrawal {
        client: Client(1),
        tx: Tx(3),
        amount: Amount(dec!(30.00)),
    });
    ledger
        .process(trx_0)
        .expect("deposits are safe on unlocked accounts");
    ledger
        .process(trx_1)
        .expect("deposits are safe on unlocked accounts");
    ledger
        .process(trx_2)
        .expect_err("should err on insufficient funds");

    assert_eq!(
        ledger.account_for(Client(1)).balance.available,
        Amount(Decimal::new(2000, 2))
    );
}
