mod hand_written;
mod label;
mod runner;
mod ungenerated;

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Not;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Written};
use graph::{Domain, Exported, Item, ItemId, ItemPath, Model, Placed, Reach, Region, Resolved};
use lint_layout::{Name, Word, words};

pub use hand_written::HandWrittenLaws;
use label::{Label, Law};
use runner::Runner;
pub use ungenerated::UngeneratedLaws;

#[derive(Debug)]
pub struct LawRunners<'model> {
    runners: Vec<Runner<'model>>,
    ungenerated: BTreeSet<ItemId>,
}

#[derive(Debug)]
struct Labels<'model> {
    generated: BTreeMap<ItemId, &'model Path>,
    monoids: BTreeSet<ItemId>,
    semigroups: BTreeSet<ItemId>,
}

impl<'model> LawRunners<'model> {
    #[must_use]
    pub fn written(&self) -> Vec<Written> {
        self.runners.iter().filter_map(|runner| Some(Written::new(runner.file()?, Bytes::from(runner)))).collect()
    }

    #[must_use]
    pub fn paths(&self) -> BTreeSet<PathBuf> {
        self.runners.iter().filter_map(Runner::file).collect()
    }

    pub(crate) fn ungenerated(&self) -> &BTreeSet<ItemId> {
        &self.ungenerated
    }

    pub(crate) fn undeclared(&self, read: &BTreeSet<&Path>) -> Vec<&'model Path> {
        self.runners
            .iter()
            .filter(|runner| runner.file().is_none_or(|file| read.get(file.as_path()).is_none()))
            .filter_map(Runner::home)
            .collect()
    }
}

impl<'model> From<&'model Model> for LawRunners<'model> {
    fn from(model: &'model Model) -> LawRunners<'model> {
        let labels = Labels::from(model);
        let unlabelled = labels.ungenerated();
        let named: Vec<(ItemId, usize, &ItemPath)> = model
            .exports(Domain::Types)
            .filter_map(|exported| Some((exported.resolved().crate_item()?, exported)))
            .filter(|(item, exported)| {
                model.declared(*item).and_then(Item::krate) == AsRef::<[Name]>::as_ref(exported.path()).first()
            })
            .map(|(item, exported): (ItemId, Exported<'model>)| {
                (item, AsRef::<[Name]>::as_ref(exported.path()).len(), exported.path())
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let shortest: Vec<&(ItemId, usize, &ItemPath)> =
            named.chunk_by(|left, right| left.0 == right.0).filter_map(<[_]>::first).collect();
        let laws: Vec<Law<'model>> = shortest
            .iter()
            .filter_map(|(item, _, path)| labels.law(*item, path))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        LawRunners {
            runners: laws.chunk_by(|left, right| left.home() == right.home()).map(Runner::from).collect(),
            ungenerated: shortest.iter().map(|(item, _, _)| *item).filter(|item| unlabelled.contains(item)).collect(),
        }
    }
}

impl<'model> Labels<'model> {
    const ARBITRARY: [Word<'static>; 2] = words!["newtype", "Arbitrary"];
    const MONOID: [Word<'static>; 2] = words!["algebra", "Monoid"];
    const SEMIGROUP: [Word<'static>; 2] = words!["algebra", "Semigroup"];

    fn subject(wanted: Option<&Resolved>, placed: &Placed<'model, &'model graph::Implementation>) -> Option<ItemId> {
        wanted.filter(|found| placed.entity().implemented() == Some(*found))?;
        placed.entity().subject_item()
    }

    fn law(&self, item: ItemId, path: &'model ItemPath) -> Option<Law<'model>> {
        let root = self.generated.get(&item)?;
        Some(Law::new(root, path, self.label(item)))
    }

    fn label(&self, item: ItemId) -> Label {
        let semigroup = self.semigroups.get(&item).map(|_| Label::Semigroup);
        self.monoids.get(&item).map(|_| Label::Monoid).or(semigroup).unwrap_or(Label::Generated)
    }

    fn ungenerated(&self) -> BTreeSet<ItemId> {
        self.monoids.union(&self.semigroups).filter(|item| self.generated.contains_key(item).not()).copied().collect()
    }
}

impl<'model> From<&'model Model> for Labels<'model> {
    fn from(model: &'model Model) -> Labels<'model> {
        let arbitrary = model.exported(Domain::Macros, &Labels::ARBITRARY.into_iter().collect());
        let monoid = model.exported(Domain::Types, &Labels::MONOID.into_iter().collect());
        let semigroup = model.exported(Domain::Types, &Labels::SEMIGROUP.into_iter().collect());
        let shipped: Vec<_> = model
            .implementations()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .collect();
        Labels {
            generated: shipped
                .iter()
                .filter(|placed| placed.entity().parameters().is_empty())
                .filter_map(|placed| Some((Labels::subject(arbitrary, placed)?, placed.root())))
                .collect(),
            monoids: shipped.iter().filter_map(|placed| Labels::subject(monoid, placed)).collect(),
            semigroups: shipped.iter().filter_map(|placed| Labels::subject(semigroup, placed)).collect(),
        }
    }
}
