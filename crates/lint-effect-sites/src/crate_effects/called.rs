use std::fmt;

use lint_layout::{Name, UsePath};

#[derive(Clone, Debug)]
pub(super) enum Called {
    Path(UsePath),
    Macro(Name),
}

impl fmt::Display for Called {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => write!(formatter, "{path}"),
            Self::Macro(name) => write!(formatter, "{name}!"),
        }
    }
}
