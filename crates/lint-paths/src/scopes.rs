mod scope;

use std::collections::{BTreeMap, BTreeSet};

use crate::module::Module;
use lint_layout::{Name, UsePath, Word};
use scope::Scope;

#[derive(Debug, Default)]
pub(crate) struct Scopes {
    scopes: BTreeMap<Module, Scope>,
    guards: BTreeMap<Module, BTreeSet<Name>>,
}

impl Scopes {
    pub(crate) fn declare(&mut self, module: &Module) {
        module.ancestry().into_iter().for_each(|ancestor| {
            self.scopes.entry(ancestor).or_default();
        });
    }

    pub(crate) fn scope_of(&mut self, module: &Module) -> &mut Scope {
        self.scopes.entry(module.clone()).or_default()
    }

    pub(crate) fn guard(&mut self, module: &Module, name: Option<Name>) {
        self.guards.entry(module.root()).or_default().extend(name);
    }

    pub(crate) fn scope(&self, module: &Module) -> Option<&Scope> {
        self.scopes.get(module)
    }

    pub(crate) fn renames_head(&self, module: &Module, path: &UsePath) -> bool {
        path.first().is_some_and(|head| {
            self.scope(module).is_some_and(|scope| scope.imported(head).is_some() || !scope.globs().is_empty())
        })
    }

    pub(crate) fn renamed_to(&self, finals: &[Word<'_>]) -> Vec<Name> {
        self.scopes.values().flat_map(|scope| scope.renamed_to(finals)).collect()
    }

    pub(crate) fn declares(&self, module: &Module) -> bool {
        self.scopes.contains_key(module)
    }

    pub(crate) fn guarded(&self, module: &Module, name: &Name) -> bool {
        self.guards.get(&module.root()).is_some_and(|names| names.contains(name))
    }
}
