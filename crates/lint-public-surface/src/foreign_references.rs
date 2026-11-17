mod crate_reach;

use std::collections::BTreeMap;

use syntax::{Ident, ItemUse, Macro, Path, UseLeaf, UseTree};

use super::export::Export;
use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

use crate_reach::CrateReach;

#[derive(Debug, Default)]
pub struct ForeignReferences {
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
        ForeignReferences { crates: kept.map(|(name, reach)| (name, reach.names_only())).into_iter().collect() }
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
    own: Option<CrateName>,
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
        Reading { own, reader, tested: own_tests }
    }
}

impl Reading {
    pub fn kept(&self, references: ForeignReferences) -> ForeignReferences {
        self.reader.kept(references, self.tested.clone())
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
        self.crates.get(crate_name).and_then(|reach| reach.unnamed(AsRef::<Name>::as_ref(export))).map(|_| export)
    }

    fn used(self, tree: &UseTree, own: Option<&CrateName>) -> ForeignReferences {
        let leaves = UseLeaf::of_tree(tree);
        let reached = leaves.iter().flat_map(|leaf| {
            let first = leaf.path().first().copied();
            let module = leaf.path().get(1).copied().map(Leaf::Named);
            [Some(Self::reach(leaf)), module].into_iter().flatten().map(move |reach| (first, reach))
        });
        reached.fold(self, |references, (first, reach)| references.recorded(first, reach, own))
    }

    fn reach<'tree>(leaf: &UseLeaf<'tree>) -> Leaf<'tree> {
        let named = leaf.end().named().map(|named| &named.ident);
        named.or_else(|| leaf.end().renamed().map(|renamed| &renamed.ident)).map_or(Leaf::Glob, Leaf::Named)
    }

    fn recorded(self, first: Option<&Ident>, leaf: Leaf<'_>, own: Option<&CrateName>) -> ForeignReferences {
        let mut crates = self.crates;
        first
            .map(CrateName::from)
            .filter(|crate_name| Some(crate_name) != own)
            .into_iter()
            .for_each(|crate_name| leaf.reached(crates.entry(crate_name).or_default()));
        ForeignReferences { crates }
    }
}

impl Fold for Reading {
    type Found = ForeignReferences;

    fn kinds(&self) -> &'static [Kind] {
        match self.reader {
            Reader::Shipped | Reader::Tests => &[Kind::ItemUse, Kind::Macro, Kind::Path],
            Reader::Neither => &[],
        }
    }

    fn on_item_use(
        &self,
        found: ForeignReferences,
        item: &ItemUse,
        _: &Facts<'_, ItemUse>,
    ) -> Visit<ForeignReferences> {
        Visit::new(found.used(&item.tree, self.own.as_ref()), Descend::Over)
    }

    fn on_macro(&self, found: ForeignReferences, _: &Macro, _: &Facts<'_, Macro>) -> Visit<ForeignReferences> {
        Visit::new(found, Descend::Unfold)
    }

    fn on_path(&self, found: ForeignReferences, path: &Path, _: &Facts<'_, Path>) -> Visit<ForeignReferences> {
        let pairs = path.segments.first().zip(path.segments.iter().nth(1));
        Visit::descending(pairs.into_iter().fold(found, |references, (first, second)| {
            references.recorded(Some(&first.ident), Leaf::Named(&second.ident), self.own.as_ref())
        }))
    }
}
