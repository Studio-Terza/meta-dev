mod accepted;
mod binder;
mod claims;
mod ranges;
mod statement;
mod term;

use std::collections::BTreeMap;
use std::fmt;

pub(crate) use accepted::{Accepted, Exclusion, Order, Union};
pub(crate) use binder::{Binder, Cases, Domain, Draw, Listed, Occupancy};
pub use claims::Claims;
pub(crate) use ranges::Ranges;
use statement::Statement;
pub(crate) use term::{Callee, Depth, Operator, Term};

pub(crate) type Binders = BTreeMap<Depth, Binder>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Claim {
    binders: Binders,
    subject: Term,
    accepted: Accepted,
}

impl Claim {
    pub(crate) fn stated(binders: &Binders, subject: Term, accepted: Accepted) -> Vec<Claim> {
        Statement::new(binders.clone(), subject, accepted).grounded()
    }
}

impl fmt::Display for Claim {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.subject, self.accepted)?;
        self.binders.iter().try_for_each(|(depth, binder)| write!(formatter, ", for x{depth} {binder}"))
    }
}
