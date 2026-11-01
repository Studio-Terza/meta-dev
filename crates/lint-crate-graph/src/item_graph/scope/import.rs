use lint_core::{Name, UsePath};

#[derive(Debug)]
pub struct Import {
    leaf: Name,
    path: UsePath,
}

impl Import {
    pub fn new(leaf: Name, path: UsePath) -> Import {
        Import { leaf, path }
    }

    pub fn names(&self, name: &Name) -> Option<&UsePath> {
        (self.leaf == *name).then_some(&self.path)
    }
}
