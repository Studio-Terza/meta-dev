mod crate_reach;

use std::collections::BTreeMap;

use syn::{Ident, ItemUse, Macro, Path, UsePath, UseTree};

use super::export::Export;
use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;
use lint_sweep::{Descend, Gathers, Kind, Sweep, Watch, Watching};

use crate_reach::CrateReach;

#[derive(Debug, Default)]
pub struct ForeignReferences {
    own_crate: Option<CrateName>,
    crates: BTreeMap<CrateName, CrateReach>,
}

#[derive(Clone, Copy)]
enum Leaf<'tree> {
    Named(&'tree Ident),
    Glob,
}

impl Leaf<'_> {
    fn reached(self, reach: &mut CrateReach) {
        match self {
            Leaf::Named(name) => reach.named(Name::from(name)),
            Leaf::Glob => reach.globbed(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reader {
    Shipped,
    Tests,
    Neither,
}

impl Reader {
    fn tested(mut read: ForeignReferences, tested: Option<CrateName>) -> ForeignReferences {
        let kept = tested.and_then(|crate_name| read.crates.remove_entry(&crate_name));
        ForeignReferences {
            crates: kept.map(|(name, reach)| (name, reach.names_only())).into_iter().collect(),
            ..ForeignReferences::default()
        }
    }

    fn kept(self, read: ForeignReferences, tested: Option<CrateName>) -> ForeignReferences {
        match self {
            Reader::Shipped => read,
            Reader::Tests => Reader::tested(read, tested),
            Reader::Neither => ForeignReferences::default(),
        }
    }
}

#[derive(Debug)]
pub struct Reading {
    references: ForeignReferences,
    reader: Reader,
    tested: Option<CrateName>,
}

impl From<&SourceFile> for Reading {
    fn from(file: &SourceFile) -> Reading {
        let head = file.head();
        let package = head.placement().package().root();
        let own_crate = head.binary().map_or_else(|| Some(CrateName::from(head.placement().crate_root())), |_| None);
        let own_tests = Some(package.directory().join(ForeignReferences::TESTS))
            .filter(|tests| head.placement().crate_root().directory().starts_with(tests))
            .map(|_| CrateName::from(package));
        let shipped = file.shipped().map(|_| Reader::Shipped);
        let reader = shipped.or_else(|| own_tests.as_ref().map(|_| Reader::Tests)).unwrap_or(Reader::Neither);
        let own = shipped.and(own_crate);
        Reading { references: ForeignReferences { own_crate: own, crates: BTreeMap::new() }, reader, tested: own_tests }
    }
}

impl Gathers<ForeignReferences> for Reading {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> ForeignReferences {
        self.reader.kept(self.references, self.tested)
    }
}

impl ForeignReferences {
    const TESTS: &'static str = "tests";

    pub fn absorb(mut self, other: ForeignReferences) -> ForeignReferences {
        other.crates.into_iter().for_each(|(crate_name, reach)| {
            self.crates.entry(crate_name).or_default().absorb(reach);
        });
        self
    }

    pub fn unnamed<'export>(&self, crate_name: &CrateName, export: &'export Export) -> Option<&'export Export> {
        self.crates.get(crate_name).and_then(|reach| reach.unnamed(export.name())).map(|_| export)
    }

    fn walk(&mut self, first: Option<&Ident>, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.walk_path(first, path),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.walk(first, item)),
            UseTree::Name(name) => self.record(first, Leaf::Named(&name.ident)),
            UseTree::Rename(rename) => self.record(first, Leaf::Named(&rename.ident)),
            UseTree::Glob(_) => self.record(first, Leaf::Glob),
        }
    }

    fn record_modules(&mut self, first: &Ident, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.record(Some(first), Leaf::Named(&path.ident)),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.record_modules(first, item)),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
        }
    }

    fn walk_path(&mut self, first: Option<&Ident>, path: &UsePath) {
        first
            .is_none()
            .then_some(&path.ident)
            .into_iter()
            .for_each(|crate_ident| self.record_modules(crate_ident, &path.tree));
        self.walk(Some(first.unwrap_or(&path.ident)), &path.tree);
    }

    fn record(&mut self, first: Option<&Ident>, leaf: Leaf<'_>) {
        first
            .map(CrateName::from)
            .filter(|crate_name| Some(crate_name) != self.own_crate.as_ref())
            .into_iter()
            .for_each(|crate_name| leaf.reached(self.crates.entry(crate_name).or_default()));
    }
}

impl<'ast> Watch<'ast> for Reading {
    fn kinds(&self) -> &'static [Kind] {
        match self.reader {
            Reader::Shipped | Reader::Tests => &[Kind::ItemUse, Kind::Macro, Kind::Path],
            Reader::Neither => &[],
        }
    }

    fn on_item_use(&mut self, item: &'ast ItemUse) -> Descend {
        self.references.walk(None, &item.tree);
        Descend::Over
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Unfold
    }

    fn on_path(&mut self, path: &'ast Path) -> Descend {
        path.segments.first().zip(path.segments.iter().nth(1)).into_iter().for_each(|(first, second)| {
            self.references.record(Some(&first.ident), Leaf::Named(&second.ident));
        });
        Descend::Into
    }
}
