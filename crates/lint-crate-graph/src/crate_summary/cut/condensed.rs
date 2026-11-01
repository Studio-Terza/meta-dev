mod spectrum;
mod tangle_index;

pub use tangle_index::TangleIndex;

use std::collections::{BTreeMap, BTreeSet};

use crate::crate_summary::cut::conductance::Conductance;
use crate::crate_summary::item_count::ItemCount;
use crate::item_graph::{ItemGraph, NodeIndex};
use spectrum::{Spectrum, Weight};

#[derive(Debug)]
pub struct Condensed {
    sizes: Vec<ItemCount>,
    weights: Vec<Vec<Weight>>,
}

impl Condensed {
    const EDGE_LENGTH: usize = 2;

    pub fn of_tangles(graph: &ItemGraph, tangles: &[Vec<NodeIndex>]) -> Condensed {
        let tangle_of: BTreeMap<NodeIndex, usize> = tangles
            .iter()
            .enumerate()
            .flat_map(|(tangle, members)| members.iter().map(move |member| (*member, tangle)))
            .collect();
        let crossings = graph
            .edges()
            .into_iter()
            .filter_map(|[from, to]| Some([*tangle_of.get(&from)?, *tangle_of.get(&to)?]))
            .flat_map(|[near, far]| [[near, far], [far, near]])
            .fold(BTreeMap::<[usize; Self::EDGE_LENGTH], f64>::new(), |mut crossings, pair| {
                *crossings.entry(pair).or_default() += 1.0;
                crossings
            });
        let weights = (0..tangles.len())
            .map(|row| {
                (0..tangles.len())
                    .map(|column| Weight::new(crossings.get(&[row, column]).copied().unwrap_or_default()))
                    .collect()
            })
            .collect();
        Condensed { sizes: tangles.iter().map(|members| ItemCount::of_len(members.len())).collect(), weights }
    }

    pub fn all(&self) -> BTreeSet<TangleIndex> {
        (0..self.sizes.len()).map(TangleIndex::from).collect()
    }

    pub fn islands(&self) -> Vec<BTreeSet<TangleIndex>> {
        self.all().into_iter().fold(Vec::new(), |islands: Vec<BTreeSet<TangleIndex>>, tangle| {
            if islands.iter().any(|island| island.contains(&tangle)) {
                islands
            } else {
                islands.into_iter().chain([self.reached(tangle, BTreeSet::from([tangle]))]).collect()
            }
        })
    }

    pub fn items(&self, side: &BTreeSet<TangleIndex>) -> ItemCount {
        side.iter().filter_map(|tangle| self.sizes.get(tangle.index())).copied().sum()
    }

    pub fn sweep_order(&self) -> Vec<TangleIndex> {
        let mut order: Vec<(TangleIndex, f64)> = Spectrum::new(&self.weights)
            .fiedler()
            .into_iter()
            .enumerate()
            .map(|(index, value)| (TangleIndex::from(index), value))
            .collect();
        order.sort_by(|left, right| left.1.total_cmp(&right.1));
        order.into_iter().map(|(tangle, _)| tangle).collect()
    }

    pub fn conductance(&self, side: &BTreeSet<TangleIndex>) -> Conductance {
        let inside = |index: &usize| side.contains(&TangleIndex::from(*index));
        let (within, beyond): (Vec<_>, Vec<_>) = self.weights.iter().enumerate().partition(|(row, _)| inside(row));
        let volume = |rows: &[(usize, &Vec<Weight>)]| -> f64 {
            rows.iter().flat_map(|(_, weights)| weights.iter()).map(|weight| weight.crossings()).sum()
        };
        let crossing: f64 = within
            .iter()
            .flat_map(|(_, weights)| weights.iter().enumerate())
            .filter(|(column, _)| !inside(column))
            .map(|(_, weight)| weight.crossings())
            .sum();
        Conductance::of_ratio(crossing, volume(&within).min(volume(&beyond)))
    }

    fn reached(&self, tangle: TangleIndex, seen: BTreeSet<TangleIndex>) -> BTreeSet<TangleIndex> {
        let neighbours: Vec<TangleIndex> = self
            .weights
            .get(tangle.index())
            .into_iter()
            .flat_map(|row| row.iter().enumerate().filter(|(_, weight)| weight.crossings() > 0.0))
            .map(|(other, _)| TangleIndex::from(other))
            .collect();
        neighbours.into_iter().fold(seen, |seen, other| self.grown(seen, other))
    }

    fn grown(&self, seen: BTreeSet<TangleIndex>, other: TangleIndex) -> BTreeSet<TangleIndex> {
        if seen.contains(&other) {
            seen
        } else {
            let mut grown = seen;
            grown.insert(other);
            self.reached(other, grown)
        }
    }
}
