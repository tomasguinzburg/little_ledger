use rust_decimal_macros::dec;

use crate::{
    io::input::{InputTransactionRecord, reader},
    model::{
        common::{Amount, Client, Tx},
        transaction::{Deposit, DisputeStatus, Transaction, Type},
    },
};

#[test]
fn negative_deserialization() {
    // <- testing negative numbers, malformed amounts, missing amounts, unknown types
    let input_data = "type,client,tx,amount
                    deposit,1,1,-1.2345
                    withdrawal,1,2,1.23.44
                    deposit,1,3,
                    homungus,1,1,
                    deposit,1,1,1.2345"; // <- Valid txn for control

    let mut rdr = reader(input_data.as_bytes());
    let mut txns: Vec<Transaction> = Vec::new();

    for raw_txn in rdr.deserialize::<InputTransactionRecord>().flatten() {
        if let Ok(txn) = Transaction::try_from(raw_txn) {
            txns.push(txn);
        }
    }

    assert_eq!(txns.len(), 1);
    assert_eq!(
        txns[0],
        Transaction {
            t_type: Type::Deposit(Deposit {
                amount: Amount::try_from(dec!(1.2345)).expect("non-negative constant"),
                dispute_status: DisputeStatus::default(),
            }),
            client: Client(1),
            tx: Tx(1)
        }
    );
}
