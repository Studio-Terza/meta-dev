mod frontier;
mod link;
mod spectrum;
mod tangle_index;

pub use tangle_index::TangleIndex;

use std::collections::{BTreeMap, BTreeSet};

use crate::crate_summary::cut::conductance::Conductance;
use crate::crate_summary::cut::split::Split;
use crate::crate_summary::item_count::ItemCount;
use frontier::Frontier;
use link::Link;
use lint_core::Least;
use lint_item_graph::Traversal;
use lint_item_graph::{ItemGraph, NodeIndex};
use newtype::{Counter, Newtype};
use spectrum::{Spectrum, Weight};

#[derive(Debug)]
pub struct Condensed {
    sizes: Vec<ItemCount>,
    rows: Vec<Vec<Link>>,
    origins: Vec<TangleIndex>,
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
        let rows = crossings.into_iter().fold(vec![Vec::new(); tangles.len()], |mut rows, ([row, column], count)| {
            rows.get_mut(row)
                .into_iter()
                .for_each(|links| links.push(Link::new(TangleIndex::from(column), Weight::new(count))));
            rows
        });
        Condensed {
            sizes: tangles.iter().map(|members| ItemCount::new(members.len())).collect(),
            rows,
            origins: (0..tangles.len()).map(TangleIndex::from).collect(),
        }
    }

    pub fn all(&self) -> BTreeSet<TangleIndex> {
        (0..self.sizes.len()).map(TangleIndex::from).collect()
    }

    pub fn islands(&self) -> Vec<BTreeSet<TangleIndex>> {
        let neighbours = |tangle: TangleIndex| self.row(tangle).iter().map(|link| link.target()).collect();
        Traversal::trees(self.all().into_iter().collect(), neighbours)
            .into_iter()
            .map(|island| island.into_iter().collect::<BTreeSet<TangleIndex>>())
            .filter(|island| island.iter().any(|tangle| !self.row(*tangle).is_empty()))
            .collect()
    }

    pub fn restricted(&self, island: &BTreeSet<TangleIndex>) -> Condensed {
        let local: BTreeMap<TangleIndex, TangleIndex> =
            island.iter().enumerate().map(|(position, tangle)| (*tangle, TangleIndex::from(position))).collect();
        let kept = |tangle: &TangleIndex| local.contains_key(tangle);
        Condensed {
            sizes: island.iter().map(|tangle| self.size(*tangle)).collect(),
            rows: island
                .iter()
                .map(|tangle| {
                    self.row(*tangle)
                        .iter()
                        .filter(|link| kept(&link.target()))
                        .filter_map(|link| Some(Link::new(*local.get(&link.target())?, link.weight())))
                        .collect()
                })
                .collect(),
            origins: island.iter().filter_map(|tangle| self.origins.get(tangle.index()).copied()).collect(),
        }
    }

    pub fn items(&self, side: &BTreeSet<TangleIndex>) -> ItemCount {
        ItemCount::total(side.iter().map(|tangle| self.size(*tangle)))
    }

    pub fn size(&self, tangle: TangleIndex) -> ItemCount {
        self.sizes.get(tangle.index()).copied().unwrap_or_default()
    }

    pub fn row(&self, tangle: TangleIndex) -> &[Link] {
        self.rows.get(tangle.index()).map_or(&[], Vec::as_slice)
    }

    pub fn volume(&self) -> Weight {
        self.rows.iter().flatten().map(|link| link.weight()).sum()
    }

    pub fn split(&self, inside: &BTreeSet<TangleIndex>) -> Split {
        let origin = |side: &BTreeSet<TangleIndex>| -> BTreeSet<TangleIndex> {
            side.iter().filter_map(|tangle| self.origins.get(tangle.index()).copied()).collect()
        };
        Split::new(self.conductance(inside), origin(inside), origin(&self.all().difference(inside).copied().collect()))
    }

    pub fn sweep(&self, floor: Least) -> Option<Split> {
        let order = self.sweep_order();
        let total = self.items(&self.all());
        let candidates: Vec<(Conductance, ItemCount)> = order
            .iter()
            .scan(Frontier::new(self), |frontier, tangle| {
                frontier.admit(*tangle);
                Some((frontier.admitted_conductance(), frontier.items()))
            })
            .collect();
        let best = candidates
            .iter()
            .enumerate()
            .take(order.len().saturating_sub(1))
            .filter(|(_, (_, items))| floor.allows(items.smaller_of(total).count()))
            .min_by(|(_, (left, _)), (_, (right, _))| left.total_cmp(*right))?;
        Some(self.split(&order.iter().take(best.0 + 1).copied().collect()))
    }

    fn sweep_order(&self) -> Vec<TangleIndex> {
        let mut order: Vec<(TangleIndex, f64)> = Spectrum::new(&self.rows)
            .fiedler()
            .into_iter()
            .enumerate()
            .map(|(index, value)| (TangleIndex::from(index), value))
            .collect();
        order.sort_by(|left, right| left.1.total_cmp(&right.1));
        order.into_iter().map(|(tangle, _)| tangle).collect()
    }

    pub fn conductance(&self, side: &BTreeSet<TangleIndex>) -> Conductance {
        let rows = |inside: bool| {
            self.rows.iter().enumerate().filter(move |(row, _)| side.contains(&TangleIndex::from(*row)) == inside)
        };
        let volume =
            |inside: bool| -> f64 { rows(inside).flat_map(|(_, row)| row).map(|link| link.weight().crossings()).sum() };
        let crossing: f64 = rows(true)
            .flat_map(|(_, row)| row)
            .filter(|link| !side.contains(&link.target()))
            .map(|link| link.weight().crossings())
            .sum();
        Conductance::of_ratio(crossing, volume(true).min(volume(false)))
    }
}
