use std::collections::{BTreeMap, BTreeSet};

use super::module_path::{ModuleIndex, ModulePath, Target};
use lint_core::SourceFile;
use lint_layout::UseEdges;

#[derive(Default)]
pub struct Users<'file>(BTreeMap<ModulePath<'file>, BTreeSet<ModulePath<'file>>>);

impl<'file> Users<'file> {
    pub fn record(&mut self, home: Option<ModulePath<'file>>, caller: ModulePath<'file>) {
        home.into_iter().fold(&mut self.0, |users, home| {
            users.entry(home).or_default().insert(caller);
            users
        });
    }

    pub fn add_tests(&mut self, modules: &ModuleIndex<'file>, files: &'file [SourceFile]) {
        files.iter().fold(self, |users, file| {
            users.add_test(modules, file);
            users
        });
    }

    pub fn callers_of(&self, module: &ModulePath<'file>) -> Option<&BTreeSet<ModulePath<'file>>> {
        self.0.get(module)
    }

    fn add_test(&mut self, modules: &ModuleIndex<'file>, file: &'file SourceFile) {
        let caller = ModulePath::of_file(file);
        UseEdges::each(file.syntax(), |path| {
            let home = Target::from_use(&caller, path, modules).and_then(|target| target.deepest_in(modules));
            self.record(home, caller);
        });
    }
}
