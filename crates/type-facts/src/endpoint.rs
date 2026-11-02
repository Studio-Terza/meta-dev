use std::fmt;

use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endpoint {
    Named(Name),
    Primitive(Name),
    Unit,
    Opaque,
}

impl Endpoint {
    pub fn resolved<Resolved>(
        &self,
        named: impl FnOnce(&Name) -> Resolved,
        hidden: Resolved,
        opaque: Resolved,
    ) -> Resolved {
        match self {
            Endpoint::Named(name) => named(name),
            Endpoint::Primitive(_) | Endpoint::Unit => hidden,
            Endpoint::Opaque => opaque,
        }
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Endpoint::Named(name) | Endpoint::Primitive(name) => write!(formatter, "{name}"),
            Endpoint::Unit => formatter.write_str("()"),
            Endpoint::Opaque => formatter.write_str("_"),
        }
    }
}
