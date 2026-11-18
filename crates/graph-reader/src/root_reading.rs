use std::collections::BTreeSet;
use std::iter;

use syntax::{File, Item, ItemMod};

use graph::{CrateExports, CrateRoots, Implementation, Model, Origin, Root};
use lint_layout::{CrateSource, Crates, Dependencies, Name, PackageName};

use crate::namespace::{Entries, Visibility};

#[derive(Debug)]
pub(crate) struct RootReading;

#[derive(Debug)]
struct RootFile;

#[derive(Debug)]
struct Widening<'crates> {
    crates: &'crates Crates,
    seen: BTreeSet<PackageName>,
    pending: Vec<PackageName>,
}

impl Iterator for Widening<'_> {
    type Item = (PackageName, Root);

    fn next(&mut self) -> Option<(PackageName, Root)> {
        let package = iter::from_fn(|| self.pending.pop()).find(|package| self.seen.insert(package.clone()))?;
        let root = RootFile::read(self.crates, &package);
        self.pending.extend(root.packages().cloned());
        Some((package, root))
    }
}

impl RootReading {
    pub(crate) fn read(model: &Model, crates: &Crates) -> CrateRoots {
        let wanted: Vec<PackageName> =
            model.implementations().filter_map(|placed| Self::derived(placed.entity())).collect();
        Widening { crates, seen: BTreeSet::new(), pending: wanted }.collect()
    }

    fn derived(implementation: &Implementation) -> Option<PackageName> {
        Some(implementation)
            .filter(|derived| derived.origin() == Origin::Derived)
            .and_then(|derived| derived.implemented()?.external()?.package().cloned())
    }
}

impl RootFile {
    pub(crate) fn read(crates: &Crates, package: &PackageName) -> Root {
        crates.source(package).and_then(Self::parsed).unwrap_or(Root::Unread)
    }

    fn parsed(source: &CrateSource) -> Option<Root> {
        let exports = Self::exports(&syntax::parse::parse_file(source.root().as_ref()).ok()?, source.dependencies());
        Some(Root::formed(source.form(), exports))
    }

    fn exports(file: &File, dependencies: &Dependencies) -> CrateExports {
        let modules: Vec<Name> = file.items.iter().filter_map(Self::module).collect();
        Entries::of_root(file).crate_exports(dependencies, &modules)
    }

    fn module(item: &Item) -> Option<Name> {
        match item {
            Item::Mod(module) => Self::visible(module),
            _ => None,
        }
    }

    fn visible(module: &ItemMod) -> Option<Name> {
        (Visibility::from(&module.vis) == Visibility::Visible).then(|| Name::from(&module.ident))
    }
}
