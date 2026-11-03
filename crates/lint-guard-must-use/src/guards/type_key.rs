use std::collections::BTreeMap;

use lint_layout::Name;
use lint_paths::Module;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeKey<'file> {
    module: Module<'file>,
    name: Name,
}

impl<'file> TypeKey<'file> {
    pub fn new(module: Module<'file>, name: Name) -> TypeKey<'file> {
        TypeKey { module, name }
    }

    pub fn type_name(&self) -> &Name {
        &self.name
    }

    pub fn aliased(&self, aliases: &BTreeMap<TypeKey<'file>, Name>) -> TypeKey<'file> {
        let name = aliases.get(self).unwrap_or(&self.name).clone();
        TypeKey { module: self.module.clone(), name }
    }

    pub fn sharing_crate<'other>(&self, other: &'other TypeKey<'file>) -> Option<&'other TypeKey<'file>> {
        (self.module.root() == other.module.root()).then_some(other)
    }
}
