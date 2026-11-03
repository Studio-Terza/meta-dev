use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub enum Finding {
    FreeFn { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::FreeFn { name } = self;
        write!(formatter, "fn {name}")
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        let Finding::FreeFn { .. } = self;
        Fix::new("make it an associated function or a method")
    }
}
