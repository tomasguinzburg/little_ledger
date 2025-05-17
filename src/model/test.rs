use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::model::common::DisputeStatus;

use super::{
    common::{Amount, Client, Tx},
    ledger::Ledger,
    transaction::{Deposit, Transaction, Type, Withdrawal},
};

#[test]
fn test_withdrawal_creation() {
    let txn = Transaction {
        t_type: Type::Withdrawal(Withdrawal {
            amount: Amount(dec!(3.14)),
        }),
        tx: Tx(1),
        client: Client(1),
    };

    assert_eq!(txn.client, Client(1));

    if let Type::Withdrawal(w) = txn.t_type {
        assert_eq!(w.amount, Amount(dec!(3.14)));
    }
}

#[test]
fn test_ledger() {
    let mut ledger = Ledger::default();
    let trxs = [
        Transaction {
            t_type: Type::Deposit(Deposit {
                amount: Amount(dec!(10.00)),
                dispute_status: DisputeStatus::Closed,
            }),
            client: Client(1),
            tx: Tx(1),
        },
        Transaction {
            t_type: Type::Deposit(Deposit {
                amount: Amount(dec!(10.00)),
                dispute_status: DisputeStatus::Closed,
            }),
            client: Client(1),
            tx: Tx(2),
        },
        Transaction {
            t_type: Type::Withdrawal(Withdrawal {
                amount: Amount(dec!(5.00)),
            }),
            client: Client(1),
            tx: Tx(3),
        },
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

    let trx_0 = Transaction {
        t_type: Type::Deposit(Deposit {
            amount: Amount(dec!(10.00)),
            dispute_status: DisputeStatus::Closed,
        }),
        client: Client(1),
        tx: Tx(1),
    };

    let trx_1 = Transaction {
        t_type: Type::Deposit(Deposit {
            amount: Amount(dec!(10.00)),
            dispute_status: DisputeStatus::Closed,
        }),
        client: Client(1),
        tx: Tx(2),
    };

    let trx_2 = Transaction {
        t_type: Type::Withdrawal(Withdrawal {
            amount: Amount(dec!(30.00)),
        }),
        client: Client(1),
        tx: Tx(3),
    };

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
