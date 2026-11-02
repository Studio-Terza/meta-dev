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
