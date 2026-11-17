mod crate_reach;

use std::collections::BTreeMap;

use syntax::{Ident, ItemUse, Macro, Path, UseLeaf, UseTree};

use super::export::Export;
use algebra::{Monoid, Semigroup};
use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;
use lint_sweep::{Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind, Sweep, Watching};

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
    scan: Contributing<ReferenceScan>,
    reader: Reader,
    tested: Option<CrateName>,
}

#[derive(Debug)]
pub struct ReferenceScan {
    own_crate: Option<CrateName>,
    reader: Reader,
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
        let scan = Contributing::new(ReferenceScan { own_crate: own, reader });
        Reading { scan, reader, tested: own_tests }
    }
}

impl Gathers<ForeignReferences> for Reading {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(&mut self.scan)
    }

    fn gathered(self) -> ForeignReferences {
        let read = self.scan.concluded(|_, (), references| references);
        self.reader.kept(read, self.tested)
    }
}

impl Semigroup for ForeignReferences {
    fn combine(mut self, other: ForeignReferences) -> ForeignReferences {
        other.crates.into_iter().for_each(|(crate_name, reach)| {
            self.crates.entry(crate_name).or_default().absorb(reach);
        });
        ForeignReferences { own_crate: self.own_crate.or(other.own_crate), ..self }
    }
}

impl Monoid for ForeignReferences {}

impl ForeignReferences {
    const TESTS: &'static str = "tests";

    fn owned_by(own_crate: Option<&CrateName>) -> ForeignReferences {
        ForeignReferences { own_crate: own_crate.cloned(), crates: BTreeMap::new() }
    }

    pub fn unnamed<'export>(&self, crate_name: &CrateName, export: &'export Export) -> Option<&'export Export> {
        self.crates.get(crate_name).and_then(|reach| reach.unnamed(AsRef::<Name>::as_ref(export))).map(|_| export)
    }

    fn used(&mut self, tree: &UseTree) {
        let leaves = UseLeaf::of_tree(tree);
        let reached = leaves.iter().flat_map(|leaf| {
            let first = leaf.path().first().copied();
            let module = leaf.path().get(1).copied().map(Leaf::Named);
            [Some(Self::reach(leaf)), module].into_iter().flatten().map(move |reach| (first, reach))
        });
        reached.for_each(|(first, reach)| self.record(first, reach));
    }

    fn reach<'tree>(leaf: &UseLeaf<'tree>) -> Leaf<'tree> {
        let named = leaf.end().named().map(|named| &named.ident);
        named.or_else(|| leaf.end().renamed().map(|renamed| &renamed.ident)).map_or(Leaf::Glob, Leaf::Named)
    }

    fn record(&mut self, first: Option<&Ident>, leaf: Leaf<'_>) {
        first
            .map(CrateName::from)
            .filter(|crate_name| Some(crate_name) != self.own_crate.as_ref())
            .into_iter()
            .for_each(|crate_name| leaf.reached(self.crates.entry(crate_name).or_default()));
    }
}

impl Contribute for ReferenceScan {
    type Open = ();
    type Part = ForeignReferences;

    fn kinds(&self) -> &'static [Kind] {
        match self.reader {
            Reader::Shipped | Reader::Tests => &[Kind::ItemUse, Kind::Macro, Kind::Path],
            Reader::Neither => &[],
        }
    }

    fn on_item_use(&self, item: &ItemUse, _: &Facts<'_, ItemUse>) -> Contributed<(), ForeignReferences> {
        let mut references = ForeignReferences::owned_by(self.own_crate.as_ref());
        references.used(&item.tree);
        Contributed::new((), references, Descend::Over)
    }

    fn on_macro(&self, _: &Macro, _: &Facts<'_, Macro>) -> Contributed<(), ForeignReferences> {
        Contributed::descending(Descend::Unfold)
    }

    fn on_path(&self, path: &Path, _: &Facts<'_, Path>) -> Contributed<(), ForeignReferences> {
        let pair = path.segments.first().zip(path.segments.iter().nth(1));
        Contributed::parted(pair.map_or_default(|(first, second)| {
            let mut references = ForeignReferences::owned_by(self.own_crate.as_ref());
            references.record(Some(&first.ident), Leaf::Named(&second.ident));
            references
        }))
    }
}
