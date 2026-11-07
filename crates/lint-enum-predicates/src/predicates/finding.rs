use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub enum Finding {
    BoolPredicate { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::BoolPredicate { name } = self;
        write!(formatter, "fn {name} -> bool")
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        let Self::BoolPredicate { .. } = self;
        Fix::new("return an enum or match where the answer is used")
    }
}
