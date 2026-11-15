use std::fmt;

use lint_core::{Finding, Fix};
use lint_layout::Name;

#[derive(Debug)]
pub(super) struct Recursive<'model> {
    name: &'model Name,
}

impl<'model> From<&'model Name> for Recursive<'model> {
    fn from(name: &'model Name) -> Recursive<'model> {
        Recursive { name }
    }
}

impl fmt::Display for Recursive<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} takes a syntax node and reaches itself through its calls", self.name)
    }
}

impl Finding for Recursive<'_> {
    fn fix(&self) -> Fix {
        Fix::new("read the node from a sweep hook or a shared fact, or move the walk into the syntax crate")
    }
}
