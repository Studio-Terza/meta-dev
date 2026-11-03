use std::fmt;

use lint_core::Fix;

#[derive(Debug)]
pub struct Finding;

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "clones a value")
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new("borrow it, take a slice or &str, name a lifetime, or move it")
    }
}
