use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub enum Finding {
    VecOut { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::VecOut { name } = self;
        write!(formatter, "fn {name}")
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        let Finding::VecOut { .. } = self;
        Fix::new("give the collection a type that owns its rules")
    }
}
