use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use rust_decimal_macros::dec;

use little_ledger::{
    io::{
        input::{create_csv_reader, deserialize_transactions},
        output::{create_csv_writer, serialize_ledger},
    },
    model::{
        common::{Amount, Client, Tx},
        ledger::Ledger,
        transaction::{Deposit, DisputeStatus, Transaction, Type, Withdrawal},
    },
};

#[test]
fn deserialize_apply_serialize() {
    let rdr =
        create_csv_reader(Some(PathBuf::from("./tests/input.csv"))).expect("should be readable");
    let txns: Vec<Transaction> = deserialize_transactions(Some(rdr), true)
        .expect("should deserialize")
        .collect();

    assert_eq!(txns.len(), 112);

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
    assert_eq!(
        txns[1],
        Transaction {
            t_type: Type::Withdrawal(Withdrawal {
                amount: Amount::ZERO,
            }),
            client: Client(1),
            tx: Tx(2)
        }
    );
    assert_eq!(
        txns[2],
        Transaction {
            t_type: Type::Dispute,
            client: Client(1),
            tx: Tx(1)
        }
    );
    assert_eq!(
        txns[3],
        Transaction {
            t_type: Type::Resolve,
            client: Client(1),
            tx: Tx(1)
        }
    );
    assert_eq!(
        txns[4],
        Transaction {
            t_type: Type::Dispute,
            client: Client(1),
            tx: Tx(1)
        }
    );
    assert_eq!(
        txns[5],
        Transaction {
            t_type: Type::Chargeback,
            client: Client(1),
            tx: Tx(1)
        }
    );

    let mut ledger = Ledger::default();
    txns.iter()
        .for_each(|txn| ledger.apply(*txn).expect("transactions are valid"));

    let file_writer =
        create_csv_writer(Some(PathBuf::from("./tests/output.csv"))).expect("should be readable");

    serialize_ledger(ledger, Some(file_writer), true).expect("buffer should flush");

    let file = File::open("./tests/output.csv").expect("file should open");
    let output_buf = BufReader::new(file);
    let output = String::from_utf8(output_buf.bytes().map(|x| x.unwrap()).collect())
        .expect("should be valid utf8");

    assert!(output.contains("client,available,held,total,locked"));
    assert!(output.contains("1,0.0000,0.0000,0.0000,true"));
    assert!(output.contains("2,5.4321,1.2345,6.6666,false"));
    assert!(
        output.contains("3,79228162514264337593543950335,0,79228162514264337593543950335,false")
    );
    assert!(output.contains("4,10000,0,10000,false"));
}
