use std::fmt;

use super::callee::Callee;
use lint_core::Name;

pub enum Finding {
    Forwards { name: Name, callee: Callee },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::Forwards { name, callee } = self;
        write!(formatter, "fn {name} only forwards to {callee}; expose the part or call that directly")
    }
}
