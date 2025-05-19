use rust_decimal_macros::dec;

use super::{account::Account, balance::Balance};

use super::{
    common::{Amount, Client, Tx},
    ledger::Ledger,
    transaction::{Deposit, DisputeStatus, Transaction, Type, Withdrawal},
};

// Amount tests
#[test]
fn negative_amount_allow_only_nonnegative_values() {
    Amount::try_from(dec!(-1)).expect_err("only non-negative values allowed");
}

// Balance tests
#[test]
fn positive_balance_credit() {
    let mut balance = Balance::default();
    balance.credit(Amount::TEN);
    assert_eq!(balance.available(), Amount::TEN);
    assert_eq!(balance.held(), Amount::ZERO);
}

#[test]
fn positive_balance_debit() {
    let mut balance = Balance::default();
    balance.credit(Amount::TEN);
    balance
        .debit(Amount::TWO)
        .expect("there are sufficient available funds");
    assert_eq!(balance.available(), Amount::TEN - Amount::TWO);
    assert_eq!(balance.held(), Amount::ZERO);
}

#[test]
fn positive_balance_hold() {
    let mut balance = Balance::default();
    balance.credit(Amount::TEN);
    balance
        .hold(Amount::TWO)
        .expect("there are available sufficient funds");
    assert_eq!(balance.available(), Amount::TEN - Amount::TWO);
    assert_eq!(balance.held(), Amount::TWO);
}

#[test]
fn positive_balance_release() {
    let mut balance = Balance::default();
    balance.credit(Amount::TEN);
    balance
        .hold(Amount::TWO)
        .expect("there are sufficient available funds");
    balance
        .release(Amount::ONE)
        .expect("there are sufficient funds on hold");
    assert_eq!(balance.available(), Amount::TEN - Amount::TWO + Amount::ONE);
    assert_eq!(balance.held(), Amount::TWO - Amount::ONE);
}

#[test]
fn positive_balance_reimburse() {
    let mut balance = Balance::default();
    balance.credit(Amount::TEN);
    balance
        .hold(Amount::TWO)
        .expect("there are sufficient available funds");
    balance
        .reimburse(Amount::ONE)
        .expect("there are sufficient funds on hold");
    assert_eq!(balance.available(), Amount::TEN - Amount::TWO);
    assert_eq!(balance.held(), Amount::TWO - Amount::ONE);
}

// Negative balance tests

#[test]
fn negative_balance_debit_insufficient_available_funds() {
    let mut balance = Balance::default();
    balance
        .debit(Amount::ONE)
        .expect_err("insufficient available funds");
}

#[test]
fn negative_balance_hold_insufficient_available_funds() {
    let mut balance = Balance::default();
    balance
        .hold(Amount::ONE)
        .expect_err("insufficient available funds");
}

#[test]
fn negative_balance_release_insufficient_funds_on_hold() {
    let mut balance = Balance::default();
    balance
        .release(Amount::ONE)
        .expect_err("insufficient funds on hold");
}

#[test]
fn negative_balance_reimburse_insufficient_funds_on_hold() {
    let mut balance = Balance::default();
    balance
        .reimburse(Amount::ONE)
        .expect_err("insufficient funds on hold");
}

// Ledger tests

#[test]
fn positive_ledger_routing() {
    let mut ledger = Ledger::default();

    let txns = [
        Transaction {
            t_type: Type::Deposit(Deposit {
                amount: Amount::TEN,
                dispute_status: DisputeStatus::default(),
            }),
            client: Client(1),
            tx: Tx(1),
        },
        Transaction {
            t_type: Type::Deposit(Deposit {
                amount: Amount::TWO,
                dispute_status: DisputeStatus::default(),
            }),
            client: Client(2),
            tx: Tx(2),
        },
    ];

    txns.map(|txn| ledger.apply(txn).expect("both txns are valid"));
    assert_eq!(
        ledger.get_account_for(Client(1)).balance.available(),
        Amount::TEN
    );
    assert_eq!(
        ledger.get_account_for(Client(2)).balance.available(),
        Amount::TWO
    );
}

// Account tests happy path

#[test]
fn positive_account_deposits_and_withdrawal() {
    let mut account = default_account();
    let txns = [
        deposit(Tx(1), Amount::TEN),
        deposit(Tx(2), Amount::TEN),
        withdrawal(Tx(3), Amount::TWO),
    ];
    txns.map(|txn| account.apply(txn).expect("happy path shouldn't err"));

    assert_eq!(
        account.balance.available(),
        Amount::TEN + Amount::TEN - Amount::TWO
    );
}

#[test]
fn positive_account_dispute_resolution() {
    let mut account = default_account();
    let txns = [deposit(Tx(1), Amount::TEN), dispute(Tx(1)), resolve(Tx(1))];

    assert!(account.apply(txns[0]).is_ok());
    assert_eq!(account.balance.available(), Amount::TEN);
    assert_eq!(account.balance.held(), Amount::ZERO);

    assert!(account.apply(txns[1]).is_ok());
    assert_eq!(account.balance.available(), Amount::ZERO);
    assert_eq!(account.balance.held(), Amount::TEN);

    assert!(account.apply(txns[2]).is_ok());
    assert_eq!(account.balance.available(), Amount::TEN);
    assert_eq!(account.balance.held(), Amount::ZERO);
}

#[test]
fn positive_account_dispute_chargeback() {
    let mut account = default_account();
    let txns = [
        deposit(Tx(1), Amount::TEN),
        dispute(Tx(1)),
        chargeback(Tx(1)),
    ];

    assert!(account.apply(txns[0]).is_ok());
    assert_eq!(account.balance.available(), Amount::TEN);
    assert_eq!(account.balance.held(), Amount::ZERO);

    assert!(account.apply(txns[1]).is_ok());
    assert_eq!(account.balance.available(), Amount::ZERO);
    assert_eq!(account.balance.held(), Amount::TEN);

    assert!(account.apply(txns[2]).is_ok());
    assert_eq!(account.balance.available(), Amount::ZERO);
    assert_eq!(account.balance.held(), Amount::ZERO);
}

#[test]
fn positive_account_edge_case_open_dispute_after_previous_one_resolved() {
    let mut account = default_account();
    let txns = [
        deposit(Tx(1), Amount::TEN),
        dispute(Tx(1)),
        resolve(Tx(1)),
        dispute(Tx(1)),
        chargeback(Tx(1)),
    ];

    txns.map(|txn| {
        account.apply(txn).expect("all transactions should succeed");
    });
    assert_eq!(account.balance.available(), Amount::ZERO);
    assert_eq!(account.balance.held(), Amount::ZERO);
    assert!(account.locked);
}

// Account tests unhappy paths
#[test]
fn negative_account_insufficient_funds() {
    let mut account = default_account();

    let txns = [
        deposit(Tx(1), Amount::ONE),
        deposit(Tx(2), Amount::ONE),
        withdrawal(Tx(3), Amount::TEN),
    ];

    account
        .apply(txns[0])
        .expect("deposits are safe on unlocked accounts");
    account
        .apply(txns[1])
        .expect("deposits are safe on unlocked accounts");
    account
        .apply(txns[2])
        .expect_err("should err on insufficient funds");

    assert_eq!(account.balance.available(), Amount::TWO);
}

#[test]
fn negative_account_rejects_transactions_when_locked() {
    let mut account = default_account();
    account.lock();

    let txns = [
        deposit(Tx(1), Amount::TEN),
        withdrawal(Tx(2), Amount::TEN),
        dispute(Tx(1)),
        resolve(Tx(1)),
        chargeback(Tx(1)),
    ];

    txns.map(|txn| {
        account
            .apply(txn)
            .expect_err("a locked account should reject all transactions");
    });
}

#[test]
fn negative_account_rejects_transactions_with_wrong_client() {
    let mut account = Account::new(Client(2)); //Different from the default
    let txns = [
        deposit(Tx(1), Amount::TEN),
        withdrawal(Tx(2), Amount::TEN),
        dispute(Tx(1)),
        resolve(Tx(1)),
        chargeback(Tx(1)),
    ];

    txns.map(|txn| {
        account
            .apply(txn)
            .expect_err("an account shouldn't process transactions from other clients");
    });
}

#[test]
fn negative_account_dispute_missing_deposit() {
    let mut account = default_account();
    let txns = [dispute(Tx(1))];

    txns.map(|txn| {
        account
            .apply(txn)
            .expect_err("can't process dispute on a deposit that doesn't exist");
    });
}

#[test]
fn negative_account_resolve_and_chargeback_on_undisputed_deposit() {
    let mut account = default_account();
    let txns = [
        deposit(Tx(1), Amount::TEN),
        resolve(Tx(1)),
        chargeback(Tx(1)),
    ];

    account.apply(txns[0]).expect("the deposit shouldn't fail");
    assert_eq!(account.balance.available(), Amount::TEN);

    txns.as_slice()[1..].iter().for_each(|txn| {
        account
            .apply(*txn)
            .expect_err("resolves and chargebacks should fail on undisputed deposits");
    });
}

#[test]
fn negative_account_dispute_insufficient_funds() {
    let mut account = default_account();
    let txns = [
        deposit(Tx(1), Amount::TEN),    //+10, +0
        withdrawal(Tx(2), Amount::ONE), //-1, +0
        dispute(Tx(1)),                 //-10, +10
    ];

    txns.as_slice()[0..2].iter().for_each(|txn| {
        account
            .apply(*txn)
            .expect("deposit and withdrawal shouldn't fail");
    });
    assert_eq!(account.balance.available(), Amount::TEN - Amount::ONE);

    account
        .apply(txns[2])
        .expect_err("dispute should fail due to insufficient funds");

    // The spec does not say we should lock the account
    assert!(!account.locked);
}

#[test]
fn negative_account_open_multiple_disputes_at_the_same_time() {
    let mut account = default_account();
    let txns = [deposit(Tx(1), Amount::TEN), dispute(Tx(1)), dispute(Tx(1))];

    txns.as_slice()[0..2].iter().for_each(|txn| {
        account
            .apply(*txn)
            .expect("deposit and first dispute on a deposit are fine");
    });
    assert_eq!(account.balance.available(), Amount::ZERO);
    assert_eq!(account.balance.held(), Amount::TEN);

    account
        .apply(txns[2])
        .expect_err("dispute should fail due to the deposit already being under dispute");
}

#[test]
fn negative_account_edge_case_open_dispute_after_previous_one_chargedback() {
    let mut account = default_account();
    let txns = [
        deposit(Tx(1), Amount::TEN),
        dispute(Tx(1)),
        chargeback(Tx(1)),
        dispute(Tx(1)),
    ];

    txns.as_slice()[0..3].iter().for_each(|txn| {
        account
            .apply(*txn)
            .expect("all transactions should succeed");
    });
    account
        .apply(txns[3])
        .expect_err("account should be locked, so the dispute should fail");
    assert_eq!(account.balance.available(), Amount::ZERO);
    assert_eq!(account.balance.held(), Amount::ZERO);
    assert!(account.locked);
}

// Helpers

const C1: Client = Client(1);

fn default_account() -> Account {
    Account::new(Client(1))
}

fn deposit(tx: Tx, a: Amount) -> Transaction {
    Transaction {
        t_type: Type::Deposit(Deposit {
            amount: a,
            dispute_status: DisputeStatus::default(),
        }),
        client: C1,
        tx,
    }
}

fn withdrawal(tx: Tx, a: Amount) -> Transaction {
    Transaction {
        t_type: Type::Withdrawal(Withdrawal { amount: a }),
        client: C1,
        tx,
    }
}

fn dispute(tx: Tx) -> Transaction {
    Transaction {
        t_type: Type::Dispute,
        client: C1,
        tx,
    }
}

fn resolve(tx: Tx) -> Transaction {
    Transaction {
        t_type: Type::Resolve,
        client: C1,
        tx,
    }
}

fn chargeback(tx: Tx) -> Transaction {
    Transaction {
        t_type: Type::Chargeback,
        client: C1,
        tx,
    }
}
