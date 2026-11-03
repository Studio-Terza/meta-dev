use std::collections::BTreeSet;

use super::Condensed;
use super::link::Link;
use super::spectrum::Weight;
use super::tangle_index::TangleIndex;
use crate::crate_summary::cut::conductance::Conductance;
use crate::crate_summary::item_count::ItemCount;
use newtype::{Counter, Newtype};

#[derive(Debug)]
pub struct Frontier<'condensed> {
    condensed: &'condensed Condensed,
    inside: BTreeSet<TangleIndex>,
    crossing: Weight,
    volume: Weight,
    whole: Weight,
    items: ItemCount,
}

impl<'condensed> Frontier<'condensed> {
    pub fn new(condensed: &'condensed Condensed) -> Frontier<'condensed> {
        Frontier {
            condensed,
            inside: BTreeSet::new(),
            crossing: Weight::default(),
            volume: Weight::default(),
            whole: condensed.volume(),
            items: ItemCount::default(),
        }
    }

    pub fn admit(&mut self, tangle: TangleIndex) {
        let row = self.condensed.row(tangle);
        let weight = |wanted: &dyn Fn(&TangleIndex) -> bool| -> Weight {
            row.iter().filter(|link| wanted(&link.target())).map(|link| link.weight()).sum()
        };
        let inward = weight(&|target| self.inside.contains(target));
        let held = |target: &TangleIndex| {
            Some(*target).filter(|held| *held == tangle).or_else(|| self.inside.get(target).copied())
        };
        let (_, leaving): (Vec<&Link>, Vec<&Link>) = row.iter().partition(|link| held(&link.target()).is_some());
        let outward: Weight = leaving.iter().map(|link| link.weight()).sum();
        self.crossing = self.crossing + (outward - inward);
        self.volume = self.volume + weight(&|_| true);
        self.items = self.items.plus(self.condensed.size(tangle));
        self.inside.insert(tangle);
    }

    pub fn admitted_conductance(&self) -> Conductance {
        let rest = self.whole - self.volume;
        Conductance::new(self.crossing.into_inner() / self.volume.into_inner().min(rest.into_inner()))
    }

    pub fn items(&self) -> ItemCount {
        self.items
    }
}
