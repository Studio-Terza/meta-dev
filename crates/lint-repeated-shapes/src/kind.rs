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
    pub(crate) fn of_role(role: Role) -> Kind {
        match role {
            Role::Target => Kind::Tests,
            Role::BinaryRoot | Role::LibraryRoot | Role::BuildScript | Role::Other => Kind::Shipped,
        }
    }

    pub(crate) fn joined(self, other: Kind) -> Kind {
        Some(self).filter(|kind| *kind == other).unwrap_or(Kind::Mixed)
    }

    pub(crate) fn fix(self) -> Fix {
        match self {
            Kind::Tests => Fix::new(
                "make the copies rows of one laws::table!, or state the property they share with a law macro such as laws::roundtrip! or laws::idempotent!",
            ),
            Kind::Shipped => Fix::new(
                "extract the shape once, as a standard trait such as From, TryFrom, FromStr or Display or one shared method, else a generic over an existing trait, else a derive",
            ),
            Kind::Mixed => {
                Fix::new("the test re-implements shipped logic; test through the shipped code instead of copying it")
            }
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Tests => formatter.write_str("test"),
            Kind::Shipped => formatter.write_str("shipped"),
            Kind::Mixed => formatter.write_str("test and shipped"),
        }
    }
}
