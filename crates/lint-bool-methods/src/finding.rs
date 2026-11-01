use std::fmt;

use syn::Ident;

use lint_layout::Name;

pub struct Finding(Name);

impl From<&Ident> for Finding {
    fn from(name: &Ident) -> Finding {
        Finding(Name::from(name))
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} -> bool", self.0)
    }
}
