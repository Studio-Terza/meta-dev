mod scope;

use std::collections::{BTreeMap, BTreeSet};

use crate::module::Module;
use lint_layout::{Name, UsePath, Word};
use scope::Scope;

#[derive(Debug, Default)]
pub(crate) struct Scopes<'file> {
    scopes: BTreeMap<Module<'file>, Scope>,
    guards: BTreeMap<Module<'file>, BTreeSet<Name>>,
}

impl<'file> Scopes<'file> {
    pub(crate) fn declare(&mut self, module: &Module<'file>) {
        module.ancestry().into_iter().for_each(|ancestor| {
            self.scopes.entry(ancestor).or_default();
        });
    }

    pub(crate) fn import(&mut self, module: &Module<'file>, local: Option<Name>, target: UsePath) {
        self.scopes
            .get_mut(module)
            .zip(Some((local, target)))
            .into_iter()
            .for_each(|(scope, (local, target))| scope.import(local, target));
    }

    pub(crate) fn glob(&mut self, module: &Module<'file>, prefix: UsePath) {
        self.scopes.get_mut(module).zip(Some(prefix)).into_iter().for_each(|(scope, prefix)| scope.glob(prefix));
    }

    pub(crate) fn guard(&mut self, module: &Module<'file>, name: Option<Name>) {
        self.guards.entry(module.root()).or_default().extend(name);
    }

    pub(crate) fn scope(&self, module: &Module<'file>) -> Option<&Scope> {
        self.scopes.get(module)
    }

    pub(crate) fn renames_head(&self, module: &Module<'file>, path: &UsePath) -> Option<&Scope> {
        let head = path.first()?;
        self.scope(module).filter(|scope| scope.imported(head).is_some() || !scope.globs().is_empty())
    }

    pub(crate) fn renamed_to<'scopes>(&'scopes self, finals: &[Word<'_>]) -> impl Iterator<Item = &'scopes Name> {
        self.scopes.values().flat_map(|scope| scope.renamed_to(finals))
    }

    pub(crate) fn guarded(&self, module: &Module<'file>, name: &Name) -> Option<&Name> {
        self.guards.get(&module.root())?.get(name)
    }
}
