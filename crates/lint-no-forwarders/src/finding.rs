use std::fmt;

use super::callee::Callee;
use lint_core::Fix;
use lint_layout::Name;

pub enum Finding {
    Forwards { name: Name, callee: Callee },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::Forwards { name, callee } = self;
        write!(formatter, "fn {name} only forwards to {callee}")
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        let Finding::Forwards { .. } = self;
        Fix::new("expose the part or call that directly")
    }
}
