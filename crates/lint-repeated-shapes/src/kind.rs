use std::fmt;

use lint_core::Fix;
use lint_placement::Role;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Tests,
    Shipped,
    Mixed,
}

impl Kind {
    pub(crate) const fn of_role(role: Role) -> Self {
        match role {
            Role::Target => Self::Tests,
            Role::BinaryRoot | Role::LibraryRoot | Role::BuildScript | Role::Other => Self::Shipped,
        }
    }

    pub(crate) fn joined(self, other: Self) -> Self {
        Some(self).filter(|kind| *kind == other).unwrap_or(Self::Mixed)
    }

    pub(crate) const fn fix(self) -> Fix {
        match self {
            Self::Tests => Fix::new(
                "make the copies rows of one laws::table!, or state the property they share with a law macro such as laws::roundtrip! or laws::idempotent!",
            ),
            Self::Shipped => Fix::new(
                "extract the shape once, as a standard trait such as From, TryFrom, FromStr or Display or one shared method, else a generic over an existing trait, else a derive",
            ),
            Self::Mixed => {
                Fix::new("the test re-implements shipped logic; test through the shipped code instead of copying it")
            }
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tests => formatter.write_str("test"),
            Self::Shipped => formatter.write_str("shipped"),
            Self::Mixed => formatter.write_str("test and shipped"),
        }
    }
}
