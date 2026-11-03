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

pub struct ModuleTree {
    modules: ModuleIndex,
    users: Users,
    deep_uses: Vec<Violation>,
    reexports: BTreeMap<ModulePath, ReexportCount>,
}

impl From<&[&SourceFile]> for ModuleTree {
    fn from(files: &[&SourceFile]) -> ModuleTree {
        let empty = ModuleTree {
            modules: ModuleIndex::from(files.iter().map(|file| ModulePath::of_file(file)).collect::<Vec<_>>()),
            users: Users::default(),
            deep_uses: Vec::new(),
            reexports: BTreeMap::new(),
        };
        files.iter().fold(empty, |tree, file| tree.absorbed(file))
    }
}

impl ModuleTree {
    pub fn violations(&self) -> impl Iterator<Item = Violation> + '_ {
        let most = ModuleNesting::REEXPORTS;
        let deep_uses = self.deep_uses.iter().cloned();
        let crowded = self.reexports.iter().filter(|(module, _)| module.segment_length() != 0).filter_map(
            move |(module, count)| {
                count.beyond(most).map(|count| {
                    Violation::new(Site::of_file(module.file()), Finding::CrowdedReexports { count, most })
                })
            },
        );
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

    fn absorbed(mut self, file: &SourceFile) -> ModuleTree {
        let module = ModulePath::of_file(file);
        self.reexports.insert(module.clone(), ReexportCount::of_file(file.syntax()));
        self.users = std::mem::take(&mut self.users).with_tests(&self.modules, file.test_modules());
        UseEdges::in_file(file.syntax()).iter().fold(self, |tree, path| tree.absorbing(&module, path))
    }

    fn absorbing(mut self, module: &ModulePath, path: &UsePath) -> ModuleTree {
        match ModulePath::from_use(module, path, &self.modules) {
            Some(target) => {
                let used = self.modules.deepest_containing(&target);
                self.users =
                    used.into_iter().fold(std::mem::take(&mut self.users), |users, home| users.recorded(home, module));
                self.deep_uses.extend(target.crossed_directory(module, &self.modules).map(|_| {
                    Violation::new(Site::of_file(module.file()), Finding::ReachesPastModRs { target: target.clone() })
                }));
                self
            }
            None => self,
        }
    }

    fn home_of(&self, module: &ModulePath) -> Option<ModulePath> {
        let outsiders: Vec<&ModulePath> =
            self.users.callers_of(module)?.iter().filter(|user| module.relates::<Ancestor>(user).is_none()).collect();
        let homes: BTreeSet<ModulePath> = outsiders
            .iter()
            .map(|user| {
                user.relates::<InsideSibling>(module).map(|_| user.truncated_to_length(module.segment_length()))
            })
            .collect::<Option<_>>()?;
        (homes.len() == 1).then(|| homes.into_iter().next()).flatten()
    }
}
