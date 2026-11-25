use super::module_path::ModulePath;
use algebra::{Monoid, Semigroup};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct Users<'file>(BTreeMap<ModulePath<'file>, BTreeSet<ModulePath<'file>>>);

impl Semigroup for Users<'_> {
    fn combine(self, other: Self) -> Self {
        Users(other.0.into_iter().fold(self.0, |mut users, (module, callers)| {
            users.entry(module).or_default().extend(callers);
            users
        }))
    }
}

impl Monoid for Users<'_> {}

impl<'file> Users<'file> {
    pub fn recorded(home: Option<ModulePath<'file>>, caller: ModulePath<'file>) -> Users<'file> {
        Users(home.into_iter().map(|module| (module, BTreeSet::from([caller]))).collect())
    }

    pub fn callers_of(&self, module: &ModulePath<'file>) -> Option<&BTreeSet<ModulePath<'file>>> {
        self.0.get(module)
    }
}
