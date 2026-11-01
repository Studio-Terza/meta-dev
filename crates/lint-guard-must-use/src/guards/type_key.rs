use std::collections::BTreeMap;

use lint_layout::Name;
use lint_paths::Module;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeKey {
    module: Module,
    name: Name,
}

impl TypeKey {
    pub fn new(module: Module, name: Name) -> TypeKey {
        TypeKey { module, name }
    }

    pub fn type_name(&self) -> &Name {
        &self.name
    }

    pub fn aliased(&self, aliases: &BTreeMap<TypeKey, Name>) -> TypeKey {
        let name = aliases.get(self).unwrap_or(&self.name).clone();
        TypeKey { module: self.module.clone(), name }
    }

    pub fn shares_crate(&self, other: &TypeKey) -> bool {
        self.module.root() == other.module.root()
    }
}
