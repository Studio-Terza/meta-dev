use super::module_path::ModulePath;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct Users<'file>(BTreeMap<ModulePath<'file>, BTreeSet<ModulePath<'file>>>);

impl<'file> Users<'file> {
    pub fn record(&mut self, home: Option<ModulePath<'file>>, caller: ModulePath<'file>) {
        home.into_iter().fold(&mut self.0, |users, module| {
            users.entry(module).or_default().insert(caller);
            users
        });
    }

    pub fn callers_of(&self, module: &ModulePath<'file>) -> Option<&BTreeSet<ModulePath<'file>>> {
        self.0.get(module)
    }
}
