use std::error::Error;
use std::fmt;

use store_ports::{Batch, Cause, Execute, StoreFailure};

use super::Anchored;
use crate::boundary::LinkRow;

#[derive(Debug)]
struct Unanchored {
    links: Vec<LinkRow>,
}

impl fmt::Display for Unanchored {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let links: Vec<String> = self.links.iter().map(ToString::to_string).collect();
        write!(formatter, "these stored links name a segment the store does not hold: {}", links.join("; "))
    }
}

impl Error for Unanchored {}

impl Batch for Anchored {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let links = LinkRow::unchecked(statements)?;
        links.is_empty().then_some(()).ok_or_else(|| StoreFailure::Refused(Cause::told(&Unanchored { links })))
    }
}
