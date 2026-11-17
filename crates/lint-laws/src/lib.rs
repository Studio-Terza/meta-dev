mod hand_written;
mod label;
mod runner;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Written};
use graph::{Domain, Exported, Item, ItemId, ItemPath, Model, Placed, Reach, Region, Resolved};
use lint_layout::{Name, Word, words};

pub use hand_written::HandWrittenLaws;
use label::{Label, Law};
use runner::Runner;

#[derive(Debug)]
pub struct LawRunners<'model> {
    runners: Vec<Runner<'model>>,
}

#[derive(Debug)]
struct Labels<'model> {
    generated: BTreeMap<ItemId, &'model Path>,
    monoids: BTreeSet<ItemId>,
}

impl LawRunners<'_> {
    #[must_use]
    pub fn written(&self) -> Vec<Written> {
        self.runners.iter().filter_map(|runner| Some(Written::new(runner.file()?, Bytes::from(runner)))).collect()
    }

    #[must_use]
    pub fn paths(&self) -> BTreeSet<PathBuf> {
        self.runners.iter().filter_map(Runner::file).collect()
    }
}

impl<'model> From<&'model Model> for LawRunners<'model> {
    fn from(model: &'model Model) -> LawRunners<'model> {
        let labels = Labels::from(model);
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
        let laws: Vec<Law<'model>> = named
            .chunk_by(|left, right| left.0 == right.0)
            .filter_map(<[_]>::first)
            .filter_map(|(item, _, path)| labels.law(*item, path))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        LawRunners {
            runners: laws
                .chunk_by(|left, right| AsRef::<Path>::as_ref(left) == AsRef::<Path>::as_ref(right))
                .map(Runner::from)
                .collect(),
        }
    }
}

impl<'model> Labels<'model> {
    const ARBITRARY: [Word<'static>; 2] = words!["newtype", "Arbitrary"];
    const MONOID: [Word<'static>; 2] = words!["algebra", "Monoid"];

    fn subject(wanted: Option<&Resolved>, placed: &Placed<'model, &'model graph::Implementation>) -> Option<ItemId> {
        wanted.filter(|found| placed.entity().implemented() == Some(*found))?;
        placed.entity().subject_item()
    }

    fn law(&self, item: ItemId, path: &'model ItemPath) -> Option<Law<'model>> {
        let root = self.generated.get(&item)?;
        Some(Law::new(root, path, self.monoids.get(&item).map_or(Label::Generated, |_| Label::Monoid)))
    }
}

impl<'model> From<&'model Model> for Labels<'model> {
    fn from(model: &'model Model) -> Labels<'model> {
        let arbitrary = model.exported(Domain::Macros, &Labels::ARBITRARY.into_iter().collect());
        let monoid = model.exported(Domain::Types, &Labels::MONOID.into_iter().collect());
        let shipped: Vec<_> = model
            .implementations()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .collect();
        Labels {
            generated: shipped
                .iter()
                .filter_map(|placed| Some((Labels::subject(arbitrary, placed)?, placed.root())))
                .collect(),
            monoids: shipped.iter().filter_map(|placed| Labels::subject(monoid, placed)).collect(),
        }
    }
}
