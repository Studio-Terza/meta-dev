use std::fmt;

use lint_core::Name;

pub enum Finding {
    VagueName { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::VagueName { name } = self;
        write!(formatter, "fn {name} names nothing; say what it does")
    }
}
