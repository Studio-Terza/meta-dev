use std::fmt;

use newtype::Counter;
use port_core::{Calls, OpName};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Within {
    name: OpName,
    calls: Calls,
    limit: Calls,
}

impl Within {
    pub(crate) fn new(name: OpName, calls: Calls, limit: Calls) -> Within {
        Within { name, calls, limit }
    }
}

impl fmt::Display for Within {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} of at most {}", self.name, self.calls.count(), self.limit.count())
    }
}
