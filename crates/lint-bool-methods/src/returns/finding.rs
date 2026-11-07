use std::fmt;

use syn::Ident;

use lint_core::Fix;
use lint_layout::Name;

pub struct Finding(Name);

impl From<&Ident> for Finding {
    fn from(name: &Ident) -> Self {
        Self(Name::from(name))
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} -> bool", self.0)
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new("return an enum to match on, an Option or Result carrying the value, or a proof type such as NonEmpty")
    }
}
