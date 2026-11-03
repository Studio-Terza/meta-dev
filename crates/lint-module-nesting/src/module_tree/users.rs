use std::collections::{BTreeMap, BTreeSet};

use super::module_path::{ModuleIndex, ModulePath};
use lint_core::SourceFile;
use lint_layout::UseEdges;

#[derive(Default)]
pub struct Users(BTreeMap<ModulePath, BTreeSet<ModulePath>>);

impl Users {
    #[must_use]
    pub fn recorded(self, home: Option<ModulePath>, caller: &ModulePath) -> Users {
        match home {
            Some(home) => self.with_caller(home, caller),
            None => self,
        }
    }

    fn with_caller(mut self, home: ModulePath, caller: &ModulePath) -> Users {
        self.0.entry(home).or_default().insert(caller.clone());
        self
    }

    #[must_use]
    pub fn with_tests(self, modules: &ModuleIndex, files: &[SourceFile]) -> Users {
        files.iter().fold(self, |users, file| users.with_test(modules, file))
    }

    pub fn callers_of(&self, module: &ModulePath) -> Option<&BTreeSet<ModulePath>> {
        self.0.get(module)
    }

    fn with_test(self, modules: &ModuleIndex, file: &SourceFile) -> Users {
        let caller = ModulePath::of_file(file);
        UseEdges::in_file(file.syntax()).iter().fold(self, |users, path| {
            let home =
                ModulePath::from_use(&caller, path, modules).and_then(|target| target.deepest_in(modules).cloned());
            users.recorded(home, &caller)
        })
    }
}
