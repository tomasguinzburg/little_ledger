use std::io::BufWriter;

use rust_decimal_macros::dec;

use payments::{
    io::{
        input::{InputTransactionRecord, csv_reader},
        output::{OutputAccountRecord, writer},
    },
    model::{
        common::{Amount, Client, Tx},
        ledger::Ledger,
        transaction::{Deposit, DisputeStatus, Transaction, Type, Withdrawal},
    },
};

#[test]
fn deserialize_apply_serialize() {
    // <- testing that it works with missing or extra trailing commas and unexpected amounts
    let input_data = "type,client,tx,amount
                    deposit,1,1,1.2345
                    withdrawal,1,2,0
                    dispute,1,1,
                    resolve,1,1,
                    dispute,1,1
                    chargeback,1,1,1.000,sarasa,some extra,sarasovich
                    deposit,2,1,1.2345
                    deposit,2,2,5.4321
                    dispute,2,2,";
    let mut rdr = csv_reader(input_data.as_bytes());
    let mut txns: Vec<Transaction> = Vec::new();

    for result in rdr.deserialize::<InputTransactionRecord>() {
        let txn = Transaction::try_from(result.expect("happy case should parse"))
            .expect("happy case should map");
        txns.push(txn);
    }

    assert_eq!(txns.len(), 9);

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

    let mut output_buffer = Vec::new();

    {
        let mut csv_writer = writer(BufWriter::new(&mut output_buffer));

        //The way this test is written, we need to sort the accounts to prevent flakyness.
        let mut sorted_accounts: Vec<_> = ledger.accounts.into_values().collect();
        sorted_accounts.sort_by_key(|acc| acc.client.0);

        for account_data in sorted_accounts {
            csv_writer
                .serialize(OutputAccountRecord::from(account_data))
                .expect("Serialization should not fail");
        }
        csv_writer.flush().expect("Flushing shouldn't fail");
    }

    let output_string = String::from_utf8(output_buffer).expect("Buffer should be valid UTF-8");

    assert_eq!(
        output_string,
        "client,available,held,total,locked\n1,0.0000,0.0000,0.0000,true\n2,1.2345,5.4321,6.6666,false\n"
    );
}
