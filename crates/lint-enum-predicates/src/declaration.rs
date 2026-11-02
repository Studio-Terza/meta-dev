use lint_layout::Name;
use lint_placement::{CrateRoot, ModulePath};

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

    pub fn named(&self, crate_root: &CrateRoot, name: &Name) -> Option<&Declaration> {
        (self.crate_root == *crate_root && self.name == *name).then_some(self)
    }

    pub fn in_module(&self, module: &ModulePath) -> Option<&Declaration> {
        (self.module == *module).then_some(self)
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }
}
