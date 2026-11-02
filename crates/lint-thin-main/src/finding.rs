use std::fmt;

use lint_core::Fix;

pub enum Finding {
    Logic,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::Logic = self;
        formatter.write_str("main.rs holds only fn main and imports")
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        let Finding::Logic = self;
        Fix::new("move this into the library")
    }
}
