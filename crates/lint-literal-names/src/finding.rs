use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub enum Finding {
    VagueName { name: Name },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::VagueName { name } = self;
        write!(formatter, "fn {name} names nothing")
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        let Finding::VagueName { .. } = self;
        Fix::new("say what it does")
    }
}
