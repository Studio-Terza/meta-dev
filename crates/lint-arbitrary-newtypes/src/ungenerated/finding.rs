use std::fmt;

use lint_layout::Name;

pub struct Finding(Name);

impl From<Name> for Finding {
    fn from(name: Name) -> Finding {
        Finding(name)
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} has no generator; add `Arbitrary` to its derives (`use newtype::Arbitrary`), with `#[newtype(strategy = ...)]` if its inner type has none",
            self.0
        )
    }
}
