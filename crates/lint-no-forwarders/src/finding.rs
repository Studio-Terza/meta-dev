use std::fmt;

use super::callee::Callee;
use syn::Ident;

use lint_core::Fix;

pub enum Finding<'file> {
    Forwards { name: &'file Ident, callee: Callee<'file> },
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::Forwards { name, callee } = self;
        write!(formatter, "fn {name} only forwards to {callee}")
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        let Finding::Forwards { .. } = self;
        Fix::new("expose the part or call that directly")
    }
}
