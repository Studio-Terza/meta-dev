use std::fmt;

use lint_core::{Finding, Fix};
use workspace_metadata::{CrateName, Role};

#[derive(Clone, Copy, Debug)]
pub(super) struct Unexpanded<'meta> {
    port: &'meta CrateName,
    role: Role,
}

impl<'meta> Unexpanded<'meta> {
    pub(super) fn new(port: &'meta CrateName, role: Role) -> Unexpanded<'meta> {
        Unexpanded { port, role }
    }
}

impl fmt::Display for Unexpanded<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "expands a macro of {} in a crate that does not declare [package.metadata.{}]",
            self.port, self.role
        )
    }
}

impl Finding for Unexpanded<'_> {
    fn fix(&self) -> Fix {
        Fix::new("declare the crate's role in its manifest, or move the expansion into a crate that declares it")
    }
}
