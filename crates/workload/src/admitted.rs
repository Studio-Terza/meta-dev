use std::fmt;

use newtype::Counter;
use port_core::{Calls, OpName};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Admitted {
    name: OpName,
    base: Calls,
    scaled: Calls,
}

impl Admitted {
    pub(crate) fn new(name: OpName, base: Calls, scaled: Calls) -> Admitted {
        Admitted { name, base, scaled }
    }
}

impl fmt::Display for Admitted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} from {}", self.name, self.scaled.count(), self.base.count())
    }
}
