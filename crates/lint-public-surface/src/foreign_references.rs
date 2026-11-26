mod crate_reach;

use std::collections::BTreeMap;

use syntax::{Ident, ItemUse, Macro, Path, UseLeaf, UseTree};

use super::export::Export;
use lint_core::{Joins, Macros, SourceFile};
use lint_layout::Name;
use lint_placement::CrateName;
use lint_sweep::{Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind};

use crate_reach::{CrateReach, Reached};

#[derive(Debug, Default)]
pub struct ForeignReferences {
    crates: BTreeMap<CrateName, CrateReach>,
}

#[derive(Clone, Copy)]
enum Leaf<'tree> {
    Named(&'tree Ident),
    Glob,
}

#[derive(Debug)]
pub struct Sighting {
    crate_name: CrateName,
    reached: Reached,
}

impl From<Leaf<'_>> for Reached {
    fn from(leaf: Leaf<'_>) -> Reached {
        match leaf {
            Leaf::Named(name) => Reached::Named(Name::from(name)),
            Leaf::Glob => Reached::Glob,
        }
    }
}

impl FromIterator<Sighting> for ForeignReferences {
    fn from_iter<Sightings: IntoIterator<Item = Sighting>>(sightings: Sightings) -> ForeignReferences {
        let crates =
            sightings.into_iter().fold(BTreeMap::new(), |mut crates: BTreeMap<CrateName, CrateReach>, seen| {
                crates.entry(seen.crate_name).or_default().absorb(CrateReach::from(seen.reached));
                crates
            });
        ForeignReferences { crates }
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
        let shipped = file.head().shipped().map(|_| Reader::Shipped);
        let reader = shipped.or_else(|| own_tests.as_ref().map(|_| Reader::Tests)).unwrap_or(Reader::Neither);
        let own = shipped.and(own_crate);
        let scan = Contributing::new(ReferenceScan { own_crate: own, reader });
        Reading { scan, reader, tested: own_tests }
    }
}

impl Gathers<ForeignReferences> for Reading {
    fn gathered(self, file: &SourceFile) -> ForeignReferences {
        let read = self.scan.concluded(file, |_, (), sightings| sightings.into_iter().collect());
        self.reader.kept(read, self.tested)
    }
}

impl Joins for ForeignReferences {
    fn combine(mut self, other: ForeignReferences) -> ForeignReferences {
        other.crates.into_iter().for_each(|(crate_name, reach)| {
            self.crates.entry(crate_name).or_default().absorb(reach);
        });
        self
    }
}

impl ForeignReferences {
    const TESTS: &'static str = "tests";

    pub fn unnamed<'export>(&self, crate_name: &CrateName, export: &'export Export) -> Option<&'export Export> {
        self.crates.get(crate_name).and_then(|reach| reach.unnamed(AsRef::<Name>::as_ref(export))).map(|_| export)
    }

    fn reach<'tree>(leaf: &UseLeaf<'tree>) -> Leaf<'tree> {
        let named = leaf.end().named().map(|named| &named.ident);
        named.or_else(|| leaf.end().renamed().map(|renamed| &renamed.ident)).map_or(Leaf::Glob, Leaf::Named)
    }
}

impl ReferenceScan {
    fn used(&self, tree: &UseTree) -> Vec<Sighting> {
        let leaves = UseLeaf::of_tree(tree);
        let reached = leaves.iter().flat_map(|leaf| {
            let first = leaf.path().first().copied();
            let module = leaf.path().get(1).copied().map(Leaf::Named);
            [Some(ForeignReferences::reach(leaf)), module].into_iter().flatten().map(move |reach| (first, reach))
        });
        reached.filter_map(|(first, reach)| self.recorded(first, reach)).collect()
    }

    fn pathed(&self, path: &Path) -> Option<Sighting> {
        let pair = path.segments.first().zip(path.segments.iter().nth(1));
        pair.and_then(|(first, second)| self.recorded(Some(&first.ident), Leaf::Named(&second.ident)))
    }

    fn transcribed(&self, definition: &Macro) -> Vec<Sighting> {
        let mentioned = Macros::mentioned(definition);
        let pathed: Vec<Sighting> = mentioned.paths().iter().filter_map(|path| self.pathed(path)).collect();
        pathed.combine(Joins::concat(mentioned.uses().iter().map(|tree| self.used(tree))))
    }

    fn recorded(&self, first: Option<&Ident>, leaf: Leaf<'_>) -> Option<Sighting> {
        first
            .map(CrateName::from)
            .filter(|crate_name| Some(crate_name) != self.own_crate.as_ref())
            .map(|crate_name| Sighting { crate_name, reached: Reached::from(leaf) })
    }
}

impl Contribute for ReferenceScan {
    type Open = ();
    type Part = Vec<Sighting>;

    fn kinds(&self) -> &'static [Kind] {
        match self.reader {
            Reader::Shipped | Reader::Tests => &[Kind::ItemUse, Kind::Macro, Kind::Path],
            Reader::Neither => &[],
        }
    }

    fn on_item_use(
        &self,
        item: &ItemUse,
        _: &Facts<'_, ItemUse>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Sighting>> {
        Contributed::new((), self.used(&item.tree), Descend::Over)
    }

    fn on_macro(
        &self,
        invoked: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Sighting>> {
        Contributed::new((), self.transcribed(invoked), Descend::Unfold)
    }

    fn on_path(
        &self,
        path: &Path,
        _: &Facts<'_, Path>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Sighting>> {
        Contributed::parted(self.pathed(path).into_iter().collect())
    }
}
