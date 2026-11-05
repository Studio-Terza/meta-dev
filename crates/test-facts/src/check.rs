mod calls;
mod domain;
mod product;
mod relation;
mod values;

use std::cmp::Ordering;
use std::fmt;

pub use calls::Calls;
pub use domain::{Breadth, Cases, Domain, Rows};
use product::Product;
pub use relation::Relation;
pub use values::Values;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    relation: Relation,
    values: Values,
    domain: Domain,
    calls: Calls,
}

impl Check {
    pub(crate) fn new(relation: Relation, values: Values, domain: Domain, calls: Calls) -> Check {
        Check { relation, values, domain, calls }
    }

    #[must_use]
    pub fn covers(&self, before: &Check) -> Option<&Check> {
        self.partial_cmp(before).filter(|order| order.is_ge()).map(|_| self)
    }
}

impl PartialOrd for Check {
    fn partial_cmp(&self, other: &Check) -> Option<Ordering> {
        Product::combined([
            self.relation.partial_cmp(&other.relation),
            self.values.partial_cmp(&other.values),
            self.domain.partial_cmp(&other.domain),
            self.calls.partial_cmp(&other.calls),
        ])
    }
}

impl fmt::Display for Check {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} pinning {} values over {}{}", self.relation, self.values, self.domain, self.calls)
    }
}
