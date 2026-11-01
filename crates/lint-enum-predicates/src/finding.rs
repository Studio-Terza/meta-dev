use std::fmt;

use lint_core::Name;

pub enum Finding {
    BoolPredicate { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::BoolPredicate { name } = self;
        write!(formatter, "fn {name} -> bool")
    }
}
