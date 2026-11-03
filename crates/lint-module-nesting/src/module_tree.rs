mod finding;
mod module_path;
mod users;

use std::collections::BTreeSet;

use finding::Finding;
use module_path::{Ancestor, InsideSibling, ModuleIndex, ModulePath, Target};

use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::UseEdges;
use lint_layout::UsePath;
use users::Users;

pub struct ModuleTree<'file> {
    modules: ModuleIndex<'file>,
    users: Users<'file>,
    deep_uses: Vec<Violation<'file>>,
}

impl<'file> From<&[&'file SourceFile]> for ModuleTree<'file> {
    fn from(files: &[&'file SourceFile]) -> ModuleTree<'file> {
        let empty = ModuleTree {
            modules: ModuleIndex::from(files.iter().map(|file| ModulePath::of_file(file)).collect::<Vec<_>>()),
            users: Users::default(),
            deep_uses: Vec::new(),
        };
        files.iter().fold(empty, |tree, file| tree.absorbed(file))
    }
}

impl<'file> ModuleTree<'file> {
    pub fn violations(&self) -> impl Iterator<Item = Violation<'file>> + '_ {
        let deep_uses = self.deep_uses.iter().cloned();
        self.modules
            .modules()
            .iter()
            .filter_map(|module| {
                self.home_of(module)
                    .map(|home| Violation::new(Site::of_file(module.file()), Finding::UsedFromOneSibling { home }))
            })
            .chain(deep_uses)
    }

    fn absorbed(mut self, file: &'file SourceFile) -> ModuleTree<'file> {
        let module = ModulePath::of_file(file);
        self.users.add_tests(&self.modules, file.test_modules());
        UseEdges::each(file.syntax(), |path| self.absorb(module, path));
        self
    }

    fn absorb(&mut self, module: ModulePath<'file>, path: &UsePath) {
        let target = Target::from_use(&module, path, &self.modules);
        let used = target.as_ref().and_then(|target| target.deepest_in(&self.modules));
        self.users.record(used, module);
        let crossed = target.as_ref().and_then(|target| target.crossed_directory(&module, &self.modules));
        self.deep_uses.extend(
            crossed.map(|target| Violation::new(Site::of_file(module.file()), Finding::ReachesPastModRs { target })),
        );
    }

    fn home_of(&self, module: &ModulePath<'file>) -> Option<ModulePath<'file>> {
        let outsiders: Vec<&ModulePath<'file>> =
            self.users.callers_of(module)?.iter().filter(|user| Ancestor::holds(module, user).is_none()).collect();
        outsiders
            .iter()
            .all(|user| InsideSibling::holds(user, module).is_some())
            .then(|| {
                let homes: BTreeSet<ModulePath<'file>> =
                    outsiders.iter().map(|user| user.truncated_like(module)).collect();
                (homes.len() == 1).then(|| homes.into_iter().next()).flatten()
            })
            .flatten()
    }
}
