use lint_core::{CrateRoot, ModulePath, Name};

use super::Kind;

#[derive(Debug)]
pub struct Declaration {
    crate_root: CrateRoot,
    module: ModulePath,
    name: Name,
    kind: Kind,
}

impl Declaration {
    pub fn new(crate_root: &CrateRoot, module: ModulePath, name: Name, kind: Kind) -> Declaration {
        Declaration { crate_root: crate_root.clone(), module, name, kind }
    }

    pub fn is_named(&self, crate_root: &CrateRoot, name: &Name) -> bool {
        self.crate_root == *crate_root && self.name == *name
    }

    pub fn is_in(&self, module: &ModulePath) -> bool {
        self.module == *module
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }
}
