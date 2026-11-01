use std::collections::BTreeMap;

use lint_layout::{Name, UsePath, Whole};

#[derive(Debug, Default)]
pub(crate) struct Scope {
    imports: BTreeMap<Name, UsePath>,
    globs: Vec<UsePath>,
}

impl Scope {
    pub(crate) fn import(&mut self, local: Option<Name>, target: UsePath) {
        self.imports.extend(local.map(|local| (local, target)).filter(|(_, target)| !target.segments().is_empty()));
    }

    pub(crate) fn glob(&mut self, prefix: UsePath) {
        self.globs.push(prefix);
    }

    pub(crate) fn renamed_to(&self, finals: &[&str]) -> Vec<Name> {
        self.imports
            .iter()
            .filter(|(_, target)| target.segments().last().is_some_and(|last| last.matches::<Whole>(finals)))
            .map(|(local, _)| local.clone())
            .collect()
    }

    pub(crate) fn imported(&self, name: &Name) -> Option<&UsePath> {
        self.imports.get(name)
    }

    pub(crate) fn globs(&self) -> &[UsePath] {
        &self.globs
    }
}
