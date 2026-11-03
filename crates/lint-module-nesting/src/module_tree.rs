mod finding;
mod module_path;
mod reexport_count;
mod users;

use std::collections::{BTreeMap, BTreeSet};

use super::ModuleNesting;
use finding::Finding;
use module_path::{Ancestor, InsideSibling, ModuleIndex, ModulePath};

use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::UseEdges;
use lint_layout::UsePath;
use reexport_count::ReexportCount;
use users::Users;

pub struct ModuleTree<'file> {
    modules: ModuleIndex<'file>,
    users: Users<'file>,
    deep_uses: Vec<Violation<'file>>,
    reexports: BTreeMap<ModulePath<'file>, ReexportCount>,
}

impl<'file> From<&[&'file SourceFile]> for ModuleTree<'file> {
    fn from(files: &[&'file SourceFile]) -> ModuleTree<'file> {
        let empty = ModuleTree {
            modules: ModuleIndex::from(files.iter().map(|file| ModulePath::of_file(file)).collect::<Vec<_>>()),
            users: Users::default(),
            deep_uses: Vec::new(),
            reexports: BTreeMap::new(),
        };
        files.iter().fold(empty, |tree, file| tree.absorbed(file))
    }
}

impl<'file> ModuleTree<'file> {
    pub fn violations(&self) -> impl Iterator<Item = Violation<'file>> + '_ {
        let most = ModuleNesting::REEXPORTS;
        let deep_uses = self.deep_uses.iter().cloned();
        let crowded = self
            .reexports
            .iter()
            .filter(|(module, _)| module.parent().is_some())
            .filter_map(move |(module, count)| count.beyond(most).map(|count| (module, count)))
            .map(move |(module, count)| {
                Violation::new(Site::of_file(module.file()), Finding::CrowdedReexports { count, most })
            });
        self.modules
            .modules()
            .iter()
            .filter_map(|module| {
                self.home_of(module)
                    .map(|home| Violation::new(Site::of_file(module.file()), Finding::UsedFromOneSibling { home }))
            })
            .chain(deep_uses)
            .chain(crowded)
    }

    fn absorbed(mut self, file: &'file SourceFile) -> ModuleTree<'file> {
        let module = ModulePath::of_file(file);
        self.reexports.insert(module.clone(), ReexportCount::of_file(file.syntax()));
        self.users = std::mem::take(&mut self.users).with_tests(&self.modules, file.test_modules());
        UseEdges::in_file(file.syntax()).iter().fold(self, |tree, path| tree.absorbing(&module, path))
    }

    fn absorbing(mut self, module: &ModulePath<'file>, path: &UsePath) -> ModuleTree<'file> {
        match ModulePath::from_use(module, path, &self.modules) {
            Some(target) => {
                let used = target.deepest_in(&self.modules);
                self.users = std::mem::take(&mut self.users).recorded(used.cloned(), module);
                self.deep_uses.extend(target.crossed_directory(module, &self.modules).map(|_| {
                    Violation::new(Site::of_file(module.file()), Finding::ReachesPastModRs { target: target.clone() })
                }));
                self
            }
            None => self,
        }
    }

    fn home_of(&self, module: &ModulePath<'file>) -> Option<ModulePath<'file>> {
        let outsiders: Vec<&ModulePath<'file>> =
            self.users.callers_of(module)?.iter().filter(|user| module.relates::<Ancestor>(user).is_none()).collect();
        outsiders
            .iter()
            .all(|user| user.relates::<InsideSibling>(module).is_some())
            .then(|| {
                let homes: BTreeSet<ModulePath<'file>> =
                    outsiders.iter().map(|user| user.truncated_like(module)).collect();
                (homes.len() == 1).then(|| homes.into_iter().next()).flatten()
            })
            .flatten()
    }
}
