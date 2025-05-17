use std::io::Write;

use csv::Writer;
use serde::Serialize;

use crate::model::{
    account::Account,
    common::{Amount, Client},
};

#[derive(Debug, Serialize)]
pub struct OutputAccountRecord {
    client: Client,
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

impl From<Account> for OutputAccountRecord {
    fn from(acc: Account) -> Self {
        OutputAccountRecord {
            client: acc.client,
            available: acc.balance.available,
            held: acc.balance.held,
            total: acc.balance.total(),
            locked: acc.locked,
        }
    }
}

pub fn writer<W: Write>(w: W) -> Writer<W> {
    csv::WriterBuilder::new()
        .has_headers(true)
        .flexible(false)
        .from_writer(w)
}
