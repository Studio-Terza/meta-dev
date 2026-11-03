use std::fmt;

use lint_core::Fix;
use syn::Ident;

pub enum Finding<'name> {
    ShortLifetime { name: &'name Ident },
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::ShortLifetime { name } = self;
        write!(formatter, "'{name}")
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        let Finding::ShortLifetime { .. } = self;
        Fix::new("name the lifetime with a word that says what it borrows from")
    }
}
