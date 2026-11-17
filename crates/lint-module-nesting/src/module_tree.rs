use lint_core::SourceFile;
mod finding;
mod module_path;
mod reexport_count;
mod users;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::ModuleNesting;
use finding::Finding;
use module_path::{InsideSibling, ModuleIndex, ModulePath, Target};

use algebra::{Monoid, Semigroup};
use lint_core::{Parsed, Site, Violation};
use lint_file::Concluded;
use lint_layout::{Name, UseEdges, UsePath};
use lint_placement::{CrateRoot, Krate, Level, Nest};
use lint_sweep::{Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind, Sweep, Watching};
use reexport_count::ReexportCount;
use syntax::ItemUse;
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
    uses: Contributing<UseScan>,
    module: Option<ModuleUses<'head>>,
}

impl<'head> From<Parsed<'_, 'head>> for ModuleGathering<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> ModuleGathering<'head> {
        let head = parsed.head();
        let file = <&SourceFile>::from(parsed);
        let module = file.shipped().map(|_| {
            let tests = file.test_modules().iter().map(|test| TestUses {
                segments: AsRef::<[Name]>::as_ref(test.head().placement().module()).to_vec(),
                uses: Sweep::gathered(test, Contributing::new(UseScan)).0,
            });
            ModuleUses {
                file: AsRef::<Path>::as_ref(head),
                crate_root: head.placement().crate_root(),
                segments: AsRef::<[Name]>::as_ref(head.placement().module()),
                reexports: ReexportCount::of_file(file.syntax()),
                uses: Vec::new(),
                tests: tests.collect(),
            }
        });
        ModuleGathering { uses: Contributing::new(UseScan), module }
    }
}

impl<'head> Gathers<ShippedModule<'head>> for ModuleGathering<'head> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        let module = self.module.as_ref();
        module.map_or_default(|_| Sweep::alone(&mut self.uses))
    }

    fn gathered(self) -> ShippedModule<'head> {
        let uses = self.uses.concluded(|_, (), used| used.0);
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

#[derive(Debug)]
pub struct UseScan;

impl Semigroup for UsePaths {
    fn combine(self, other: UsePaths) -> UsePaths {
        UsePaths(self.0.combine(other.0))
    }
}

impl Monoid for UsePaths {}

impl Contribute for UseScan {
    type Open = ();
    type Part = UsePaths;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemUse]
    }

    fn on_item_use(&self, item: &ItemUse, _: &Facts<'_, ItemUse>) -> Contributed<(), UsePaths> {
        Contributed::new((), UsePaths(UseEdges::of_tree(&item.tree).into_iter().collect()), Descend::Over)
    }
}

impl Gathers<UsePaths> for Contributing<UseScan> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> UsePaths {
        self.concluded(|_, (), used| used)
    }
}

impl<'file> From<&'file [ModuleUses<'file>]> for ModuleTree<'file> {
    fn from(files: &'file [ModuleUses<'file>]) -> ModuleTree<'file> {
        let empty = ModuleTree {
            modules: ModuleIndex::from(
                files
                    .iter()
                    .map(|file| ModulePath::new(file.file, Nest::module(Krate::Rooted(file.crate_root), file.segments)))
                    .collect::<Vec<_>>(),
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
            .filter(|(module, _)| AsRef::<Nest<'_>>::as_ref(*module).level() == Level::Module)
            .filter_map(move |(module, count)| count.beyond(most).map(|crowded| (module, crowded)))
            .map(move |(module, count)| {
                Violation::new(Site::of_file(module.path()), Finding::CrowdedReexports { count, most })
            });
        self.modules
            .modules()
            .iter()
            .filter_map(|module| {
                self.home_of(module).map(|home| {
                    Violation::new(
                        Site::of_file(module.path()),
                        Finding::UsedFromOneSibling { home: *AsRef::<Nest<'_>>::as_ref(&home) },
                    )
                })
            })
            .chain(deep_uses)
            .chain(crowded)
    }

    fn absorbed(mut self, file: &'file ModuleUses<'file>) -> ModuleTree<'file> {
        let module = ModulePath::new(file.file, Nest::module(Krate::Rooted(file.crate_root), file.segments));
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
            Violation::new(Site::of_file(module.path()), Finding::ReachesPastModRs { target: crossing })
        }));
    }

    fn home_of(&self, module: &ModulePath<'file>) -> Option<ModulePath<'file>> {
        let outsiders: Vec<&ModulePath<'file>> = self
            .users
            .callers_of(module)?
            .iter()
            .filter(|user| AsRef::<Nest<'_>>::as_ref(module).holding(*AsRef::<Nest<'_>>::as_ref(*user)).is_none())
            .collect();
        outsiders
            .iter()
            .all(|user| InsideSibling::holds(user, module).is_some())
            .then(|| {
                let homes: BTreeSet<ModulePath<'file>> =
                    outsiders.iter().map(|user| InsideSibling::home(user, module)).collect();
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
        let caller = ModulePath::new(Path::new(""), Nest::module(Krate::Rooted(crate_root), &self.segments));
        self.uses.iter().for_each(|path| {
            let home = Target::from_use(&caller, path, modules).and_then(|target| target.deepest_in(modules));
            users.record(home, caller);
        });
    }
}
