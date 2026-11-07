use lint_layout::Name;

use crate::predicates::Kind;

#[derive(Debug)]
pub struct Declared {
    name: Name,
    kind: Kind,
}

impl Declared {
    pub fn new(name: Name, kind: Kind) -> Self {
        Self { name, kind }
    }

    pub fn named(&self, name: &Name) -> Option<Kind> {
        (self.name == *name).then_some(self.kind)
    }
}
