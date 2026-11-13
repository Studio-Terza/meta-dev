use std::collections::BTreeMap;

use lint_layout::{Name, UsePath, Whole, Word};

#[derive(Debug, Default)]
pub(crate) struct Scope {
    imports: BTreeMap<Name, UsePath>,
    globs: Vec<UsePath>,
}

impl Scope {
    pub(crate) fn import(&mut self, local: Option<Name>, target: UsePath) {
        let named = local.zip(target.segments().first()).map(|(alias, _)| alias);
        self.imports.extend(named.map(|alias| (alias, target)));
    }

    pub(crate) fn glob(&mut self, prefix: UsePath) {
        self.globs.push(prefix);
    }

    pub(crate) fn renamed_to(&self, finals: &[Word<'_>]) -> impl Iterator<Item = &Name> {
        self.imports
            .iter()
            .filter(|(_, target)| target.segments().last().and_then(|last| last.matches::<Whole>(finals)).is_some())
            .map(|(local, _)| local)
    }

    pub(crate) fn imported(&self, name: &Name) -> Option<&UsePath> {
        self.imports.get(name)
    }

    pub(crate) fn globs(&self) -> &[UsePath] {
        &self.globs
    }
}

impl FromIterator<Scope> for Scope {
    fn from_iter<Parts: IntoIterator<Item = Scope>>(parts: Parts) -> Scope {
        parts.into_iter().fold(Scope::default(), |mut merged, part| {
            merged.imports.extend(part.imports);
            merged.globs.extend(part.globs);
            merged
        })
    }
}
