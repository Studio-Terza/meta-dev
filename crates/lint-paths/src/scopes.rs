mod scope;

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use crate::module::Module;
use lint_layout::{Name, UsePath, Word};
use lint_placement::CrateFile;
use scope::Scope;

#[derive(Debug, Default)]
pub(crate) struct Scopes {
    scopes: BTreeMap<Module, Scope>,
    guards: BTreeMap<CrateFile, BTreeSet<Name>>,
}

impl Scopes {
    pub(crate) fn declare(&mut self, module: &Module) {
        iter::successors(Some(module.clone()), Module::parent).for_each(|ancestor| {
            self.scopes.entry(ancestor).or_default();
        });
    }

    pub(crate) fn import(&mut self, module: &Module, local: Option<Name>, target: UsePath) {
        self.scopes.entry(module.clone()).or_default().import(local, target);
    }

    pub(crate) fn glob(&mut self, module: &Module, prefix: UsePath) {
        self.scopes.entry(module.clone()).or_default().glob(prefix);
    }

    pub(crate) fn guard(&mut self, module: &Module, name: Option<Name>) {
        self.guards.entry(AsRef::<CrateFile>::as_ref(module).clone()).or_default().extend(name);
    }

    pub(crate) fn scope(&self, module: &Module) -> Option<&Scope> {
        self.scopes.get(module)
    }

    pub(crate) fn renames_head(&self, module: &Module, path: &UsePath) -> Option<&Scope> {
        let head = path.segments().first()?;
        self.scope(module).filter(|scope| scope.imported(head).is_some() || !scope.globs().is_empty())
    }

    pub(crate) fn renamed_to(&self, finals: &[Word<'_>]) -> Vec<Name> {
        self.scopes.values().flat_map(|scope| scope.renamed_to(finals)).collect()
    }

    pub(crate) fn guarded(&self, module: &Module, name: &Name) -> Option<&Name> {
        self.guards.get(AsRef::<CrateFile>::as_ref(module))?.get(name)
    }
}
