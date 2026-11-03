use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

#[derive(Debug)]
pub(super) struct Finding<'name> {
    repeat: &'name Name,
    first: &'name Name,
}

impl<'name> Finding<'name> {
    pub(super) fn new(repeat: &'name Name, first: &'name Name) -> Finding<'name> {
        Finding { repeat, first }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "test {} differs from {} only in literals", self.repeat, self.first)
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "make them rows of one laws::table!, or state the property with a law macro such as laws::roundtrip! or laws::idempotent!",
        )
    }
}
