use std::fmt;

use lint_core::Name;

pub enum Finding {
    VecOut { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::VecOut { name } = self;
        write!(formatter, "fn {name}")
    }
}
