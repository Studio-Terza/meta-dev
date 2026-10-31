use std::path::{Path, PathBuf};

use lint_core::{ModulePath, Name};

use super::Kind;

#[derive(Debug)]
pub struct Declaration {
    crate_root: PathBuf,
    module: ModulePath,
    name: Name,
    kind: Kind,
}

impl Declaration {
    pub fn new(crate_root: &Path, module: ModulePath, name: Name, kind: Kind) -> Declaration {
        Declaration { crate_root: crate_root.to_path_buf(), module, name, kind }
    }

    pub fn is_named(&self, crate_root: &Path, name: &Name) -> bool {
        self.crate_root == crate_root && self.name == *name
    }

    pub fn is_in(&self, module: &ModulePath) -> bool {
        self.module == *module
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }
}
