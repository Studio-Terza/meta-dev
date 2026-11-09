use chrono::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Stamp {
    at: NaiveDate,
    note: String,
}
