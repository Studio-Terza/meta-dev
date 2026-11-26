use algebra::Semigroup;
use lint_core::Fix;
use lint_placement::Role;
use strum::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    #[strum(serialize = "test")]
    Tests,
    #[strum(serialize = "shipped")]
    Shipped,
    #[strum(serialize = "test and shipped")]
    Mixed,
}

impl Semigroup for Kind {
    fn combine(self, other: Kind) -> Kind {
        Some(self).filter(|kind| *kind == other).unwrap_or(Kind::Mixed)
    }
}

impl Kind {
    pub(crate) fn of_role(role: Role) -> Kind {
        match role {
            Role::Target => Kind::Tests,
            Role::BinaryRoot | Role::LibraryRoot | Role::BuildScript | Role::Other => Kind::Shipped,
        }
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
