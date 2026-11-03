use lint_layout::Name;

use crate::Kind;

#[derive(Debug)]
pub struct Declared {
    name: Name,
    kind: Kind,
}

impl Declared {
    pub fn new(name: Name, kind: Kind) -> Declared {
        Declared { name, kind }
    }

    pub fn named(&self, name: &Name) -> Option<Kind> {
        (self.name == *name).then_some(self.kind)
    }
}
