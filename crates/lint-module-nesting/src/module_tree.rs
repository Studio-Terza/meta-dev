mod finding;
mod module_path;
mod reexport_count;
mod users;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::ModuleNesting;
use finding::Finding;
use module_path::{Ancestor, InsideSibling, ModuleIndex, ModulePath, Target};

use lint_core::{Parsed, Site, Violation};
use lint_file::Concluded;
use lint_layout::{Name, UseEdges, UsePath};
use lint_placement::CrateRoot;
use lint_sweep::{Descend, Gathers, Kind, Sweep, Watch, Watching};
use reexport_count::ReexportCount;
use syn::ItemUse;
use users::Users;

pub struct ModuleTree<'file> {
    modules: ModuleIndex<'file>,
    users: Users<'file>,
    deep_uses: Vec<Violation<'file>>,
    reexports: BTreeMap<ModulePath<'file>, ReexportCount>,
}

#[derive(Debug)]
pub struct TestUses {
    segments: Vec<Name>,
    uses: Vec<UsePath>,
}

#[derive(Debug)]
pub struct ModuleUses<'head> {
    file: &'head Path,
    crate_root: &'head CrateRoot,
    segments: &'head [Name],
    reexports: ReexportCount,
    uses: Vec<UsePath>,
    tests: Vec<TestUses>,
}

#[derive(Debug)]
pub struct ShippedModule<'head>(Option<ModuleUses<'head>>);

impl<'head> IntoIterator for ShippedModule<'head> {
    type Item = ModuleUses<'head>;
    type IntoIter = std::option::IntoIter<ModuleUses<'head>>;

    fn into_iter(self) -> std::option::IntoIter<ModuleUses<'head>> {
        self.0.into_iter()
    }
}

#[derive(Debug)]
pub struct ModuleGathering<'head> {
    uses: UsePaths,
    module: Option<ModuleUses<'head>>,
}

impl<'head> From<Parsed<'_, 'head>> for ModuleGathering<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> ModuleGathering<'head> {
        let head = parsed.head();
        let file = parsed.file();
        let module = file.shipped().map(|_| {
            let tests = file.test_modules().iter().map(|test| TestUses {
                segments: test.head().placement().module().segments().to_vec(),
                uses: Sweep::gathered(test, UsePaths::default()),
            });
            ModuleUses {
                file: head.path(),
                crate_root: head.placement().crate_root(),
                segments: head.placement().module().segments(),
                reexports: ReexportCount::of_file(file.syntax()),
                uses: Vec::new(),
                tests: tests.collect(),
            }
        });
        ModuleGathering { uses: UsePaths::default(), module }
    }
}

impl<'head> Gathers<ShippedModule<'head>> for ModuleGathering<'head> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        let module = self.module.as_ref();
        module.map_or_default(|_| Sweep::alone(&mut self.uses))
    }

    fn gathered(self) -> ShippedModule<'head> {
        let uses = self.uses.0;
        ShippedModule(self.module.map(|module| ModuleUses { uses, ..module }))
    }
}

impl<'head> FromIterator<ShippedModule<'head>> for Concluded<'head> {
    fn from_iter<Parts: IntoIterator<Item = ShippedModule<'head>>>(parts: Parts) -> Concluded<'head> {
        let modules: Vec<ModuleUses<'head>> = parts.into_iter().flatten().collect();
        let heads: BTreeMap<&Path, &'head Path> = modules.iter().map(|module| (module.file, module.file)).collect();
        let tree = ModuleTree::from(modules.as_slice());
        tree.violations()
            .filter_map(|violation| heads.get(violation.site().path()).map(|head| violation.moved(head)))
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct UsePaths(Vec<UsePath>);

impl<'ast> Watch<'ast> for UsePaths {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemUse]
    }

    fn on_item_use(&mut self, item: &'ast ItemUse) -> Descend {
        self.0.extend(UseEdges::of_tree(&item.tree));
        Descend::Over
    }
}

impl Gathers<Vec<UsePath>> for UsePaths {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Vec<UsePath> {
        self.0
    }
}

impl<'file> From<&'file [ModuleUses<'file>]> for ModuleTree<'file> {
    fn from(files: &'file [ModuleUses<'file>]) -> ModuleTree<'file> {
        let empty = ModuleTree {
            modules: ModuleIndex::from(
                files.iter().map(|file| ModulePath::new(file.crate_root, file.segments, file.file)).collect::<Vec<_>>(),
            ),
            users: Users::default(),
            deep_uses: Vec::new(),
            reexports: BTreeMap::new(),
        };
        files.iter().fold(empty, ModuleTree::absorbed)
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
            .filter_map(move |(module, count)| count.beyond(most).map(|crowded| (module, crowded)))
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

    fn absorbed(mut self, file: &'file ModuleUses<'file>) -> ModuleTree<'file> {
        let module = ModulePath::new(file.crate_root, file.segments, file.file);
        self.reexports.insert(module, file.reexports);
        file.tests.iter().for_each(|test| test.recorded(&mut self.users, &self.modules, file.crate_root));
        file.uses.iter().for_each(|path| self.absorb(module, path));
        self
    }

    fn absorb(&mut self, module: ModulePath<'file>, path: &UsePath) {
        let target = Target::from_use(&module, path, &self.modules);
        let used = target.as_ref().and_then(|found| found.deepest_in(&self.modules));
        self.users.record(used, module);
        let crossed = target.as_ref().and_then(|found| found.crossed_directory(&module, &self.modules));
        self.deep_uses.extend(crossed.map(|crossing| {
            Violation::new(Site::of_file(module.file()), Finding::ReachesPastModRs { target: crossing })
        }));
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

impl TestUses {
    fn recorded<'file>(
        &'file self,
        users: &mut Users<'file>,
        modules: &ModuleIndex<'file>,
        crate_root: &'file CrateRoot,
    ) {
        let caller = ModulePath::new(crate_root, &self.segments, Path::new(""));
        self.uses.iter().for_each(|path| {
            let home = Target::from_use(&caller, path, modules).and_then(|target| target.deepest_in(modules));
            users.record(home, caller);
        });
    }
}
