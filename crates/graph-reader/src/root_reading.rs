use std::collections::BTreeSet;

use syn::{File, Item, ItemMod};

use graph::{CrateExports, CrateRoots, Implementation, Model, Origin, Root};
use lint_layout::{CrateSource, Crates, Dependencies, Name, PackageName};

use crate::namespace::{Entries, Visibility};

#[derive(Debug)]
pub(crate) struct RootReading;

#[derive(Debug)]
struct RootFile;

#[derive(Debug)]
struct Closing {
    read: CrateRoots,
    pending: BTreeSet<PackageName>,
}

impl RootReading {
    pub(crate) fn read(model: &Model, crates: &Crates) -> CrateRoots {
        let wanted: BTreeSet<PackageName> =
            model.implementations().filter_map(|placed| Self::derived(placed.entity())).collect();
        Closing { read: CrateRoots::default(), pending: wanted }.closed(crates)
    }

    fn derived(implementation: &Implementation) -> Option<PackageName> {
        Some(implementation)
            .filter(|derived| derived.origin() == Origin::Derived)
            .and_then(|derived| derived.implemented()?.external()?.package().cloned())
    }
}

impl Closing {
    fn closed(self, crates: &Crates) -> CrateRoots {
        match self.pending.first().map(|_| ()) {
            None => self.read,
            Some(()) => self.widened(crates).closed(crates),
        }
    }

    fn widened(self, crates: &Crates) -> Closing {
        let fresh: BTreeSet<PackageName> =
            self.pending.into_iter().filter(|package| self.read.root(package).is_none()).collect();
        let read = fresh
            .iter()
            .fold(self.read, |roots, package| roots.with_root(package.clone(), RootFile::read(crates, package)));
        let pending = fresh.iter().filter_map(|package| read.root(package)).flat_map(Root::packages).cloned().collect();
        Closing { read, pending }
    }
}

impl RootFile {
    pub(crate) fn read(crates: &Crates, package: &PackageName) -> Root {
        crates.source(package).and_then(Self::parsed).unwrap_or(Root::Unread)
    }

    fn parsed(source: &CrateSource) -> Option<Root> {
        let exports = Self::exports(&syn::parse_file(source.root().as_ref()).ok()?, source.dependencies());
        Some(Root::formed(source.form(), exports))
    }

    fn exports(file: &File, dependencies: &Dependencies) -> CrateExports {
        let modules: Vec<Name> = file.items.iter().filter_map(Self::module).collect();
        Entries::of_items(&file.items).crate_exports(dependencies, &modules)
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
