use std::fmt;

use syn::Ident;

use lint_core::Fix;

pub enum Finding<'file> {
    FreeText { context: &'file Ident },
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding::FreeText { context } = self;
        write!(formatter, "free text in {context}")
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        let Finding::FreeText { .. } = self;
        Fix::new(
            "split it into typed parts, or use a newtype that validates (serde try_from, or a Deserialize that calls FromStr)",
        )
    }
}
