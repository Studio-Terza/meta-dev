use std::error::Error;
use std::fmt;

use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, Statements, StoreFailure, Version, Written};

use crate::boundary::LinkRow;

#[derive(Debug, Op)]
pub struct Anchored;

#[derive(Debug)]
struct Unanchored {
    links: Vec<LinkRow>,
}

impl Anchored {
    pub(super) const PENDING: Version = Version::new(20_261_109_000_001);
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

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let links = LinkRow::unchecked(statements)?;
        links.is_empty().then(Written::zero).ok_or_else(|| StoreFailure::Refused(Box::new(Unanchored { links })))
    }
}
