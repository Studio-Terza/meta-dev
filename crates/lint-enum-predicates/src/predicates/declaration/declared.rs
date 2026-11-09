use lint_layout::Name;

use crate::predicates::Kind;

#[derive(Debug)]
pub struct Declared {
    name: Name,
    kind: Kind,
}

impl Declared {
    pub fn new(name: Name, kind: Kind) -> Declared {
        Declared { name, kind }
    }
}

impl AsRef<Name> for Declared {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl From<&Declared> for Kind {
    fn from(declared: &Declared) -> Kind {
        declared.kind
    }
}
