use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

pub struct Finding(Name);

impl From<Name> for Finding {
    fn from(name: Name) -> Self {
        Self(name)
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} has no generator", self.0)
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new(
            "add `Arbitrary` to its derives (`use newtype::Arbitrary`), with `#[newtype(strategy = ...)]` if its inner type has none",
        )
    }
}
