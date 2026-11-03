use std::fmt;

use lint_core::Fix;
use syn::Ident;

pub enum Finding<'name> {
    FreeFn { name: &'name Ident },
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::FreeFn { name } = self;
        write!(formatter, "fn {name}")
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        let Finding::FreeFn { .. } = self;
        Fix::new("make it an associated function or a method")
    }
}
