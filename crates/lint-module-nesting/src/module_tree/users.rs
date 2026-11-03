use std::collections::{BTreeMap, BTreeSet};

use super::module_path::{ModuleIndex, ModulePath};
use lint_core::SourceFile;
use lint_layout::UseEdges;

#[derive(Default)]
pub struct Users<'file>(BTreeMap<ModulePath<'file>, BTreeSet<ModulePath<'file>>>);

impl<'file> Users<'file> {
    #[must_use]
    pub fn recorded(self, home: Option<ModulePath<'file>>, caller: &ModulePath<'file>) -> Users<'file> {
        match home {
            Some(home) => self.with_caller(home, caller),
            None => self,
        }
    }

    fn with_caller(mut self, home: ModulePath<'file>, caller: &ModulePath<'file>) -> Users<'file> {
        self.0.entry(home).or_default().insert(caller.clone());
        self
    }

    pub fn callers_of(&self, module: &ModulePath<'file>) -> Option<&BTreeSet<ModulePath<'file>>> {
        self.0.get(module)
    }

    #[must_use]
    pub fn with_test(self, modules: &ModuleIndex<'file>, file: &'file SourceFile) -> Users<'file> {
        let caller = ModulePath::of_file(file);
        UseEdges::in_file(file.syntax()).iter().fold(self, |users, path| {
            let home =
                ModulePath::from_use(&caller, path, modules).and_then(|target| modules.deepest(&target).cloned());
            users.recorded(home, &caller)
        })
    }
}
