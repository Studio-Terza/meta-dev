use crate::{Cursor, Record};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Read {
    pub records: Vec<Record>,
    pub cursor: Cursor,
}
