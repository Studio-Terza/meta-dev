use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Finding<'model> {
    context: &'model Name,
}

impl<'model> Finding<'model> {
    pub(crate) fn new(context: &'model Name) -> Finding<'model> {
        Finding { context }
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "split it into typed parts, or use a newtype that validates (#[newtype(checked)] with an impl of newtype::Checked)",
        )
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "free text in {}", self.context)
    }
}
