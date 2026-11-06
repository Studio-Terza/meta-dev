use std::fmt;

use lint_core::{Finding, Fix};

#[derive(Debug)]
pub(super) struct CountedName;

impl fmt::Display for CountedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("names counted_call or counted_acquire outside port-core and port-derive")
    }
}

impl Finding for CountedName {
    fn fix(&self) -> Fix {
        Fix::new(
            "declare the trait with #[port] and implement its Filler, so the generated code counts each call and acquisition",
        )
    }
}
