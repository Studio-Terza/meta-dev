use std::fmt;

use lint_core::{Finding, Fix};

#[derive(Debug)]
pub(crate) struct Unformatted;

impl fmt::Display for Unformatted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a store_ports::query! writes its SQL other than as sqlformat formats it")
    }
}

impl Finding for Unformatted {
    fn fix(&self) -> Fix {
        Fix::new(
            "write the SQL as sqlformat formats it with its default options, starting at column 0 on the line after r#\" and ending with a line break before \"#",
        )
    }
}
