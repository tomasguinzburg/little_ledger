use rust_decimal_macros::dec;

use crate::{
    io::{self, input::InputTransactionRecord},
    model::{
        common::{Amount, Client, Tx},
        transaction::{Deposit, Transaction, Withdrawal},
    },
};

#[test]
fn test_csv_transaction_deserialization() {
    let csv_data = "type,client,tx,amount
                    deposit,1,100,10.50
                    withdrawal,2,200,5.25
                    deposit,5,500,
                    withdrawal,6,600,
                    unknown_type,10,1000,5.0
                    deposit,11,1100,invalid_amount
                    deposit,12,not_a_client,12.0
                    withdrawal,not_a_client_either,1300,13.0
                    deposit,14,1400,20.0,extra_column";

    let mut rdr = io::input::reader(csv_data.as_bytes());

    let mut txns: Vec<Transaction> = Vec::new();
    let mut csverrs: Vec<String> = Vec::new();
    let mut ierrs: Vec<String> = Vec::new();

    for (line_num, result) in rdr.deserialize::<InputTransactionRecord>().enumerate() {
        let actual_line_num = line_num + 2;
        match result {
            Ok(raw_rec) => match Transaction::try_from(raw_rec) {
                Ok(txn) => txns.push(txn),
                Err(e) => ierrs.push(format!("Line {actual_line_num}: {e}")),
            },
            Err(e) => {
                csverrs.push(format!("Line {actual_line_num}: {e}"));
            }
        }
    }

    assert_eq!(txns.len(), 3, "Expected 3 succ. Got: {txns:?}",);

    assert_eq!(
        txns[0],
        Transaction::Deposit(Deposit {
            client: Client(1),
            tx: Tx(100),
            amount: Amount(dec!(10.50))
        })
    );
    assert_eq!(
        txns[1],
        Transaction::Withdrawal(Withdrawal {
            client: Client(2),
            tx: Tx(200),
            amount: Amount(dec!(5.25))
        })
    );
    assert_eq!(
        txns[2],
        Transaction::Deposit(Deposit {
            // From "deposit,14,1400,20.0,extra_column"
            client: Client(14),
            tx: Tx(1400),
            amount: Amount(dec!(20.0))
        })
    );
    assert_eq!(ierrs.len(), 2, "Expected 2 input_errs. Got: {ierrs:?}",);
    assert!(
        ierrs
            .iter()
            .any(|e| e.contains("Line 4: missing mandatory amount")),
        "Line 4: missing mandatory amount not found. Input errors: {ierrs:?}",
    );
    assert!(
        ierrs
            .iter()
            .any(|e| e.contains("Line 5: missing mandatory amount")),
        "Line 5: missing mandatory amount not found. Input errors: {ierrs:?}",
    );
    assert_eq!(
        csverrs.len(),
        4,
        "Expected 4 CSV deserialization errors. Got: {csverrs:?}",
    );
    assert!(
        csverrs
            .iter()
            .any(|e| e.contains("Line 6") && e.contains("unknown_type")),
        "Error for unknown_type not found. Errors: {csverrs:?}",
    );
    assert!(
        csverrs
            .iter()
            .any(|e| e.contains("Line 7") && e.contains("invalid_amount")),
        "Error for invalid_amount not found. Errors: {csverrs:?}",
    );
    assert!(
        csverrs
            .iter()
            .any(|e| e.contains("Line 8") && e.contains("invalid digit found in string")),
        "Error for not_a_client (1) not found. Errors: {csverrs:?}",
    );
    assert!(
        csverrs
            .iter()
            .any(|e| e.contains("Line 9") && e.contains("invalid digit found in string")),
        "Error for not_a_client (2) not found. Errors: {csverrs:?}",
    );
}
