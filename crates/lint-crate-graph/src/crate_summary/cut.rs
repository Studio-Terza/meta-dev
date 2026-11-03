mod condensed;
mod conductance;
mod split;

use std::collections::BTreeSet;
use std::fmt;

use crate::crate_summary::cut::conductance::Conductance;
use condensed::{Condensed, TangleIndex};
use lint_core::Least;
use lint_item_graph::{ItemGraph, NodeIndex};
use lint_layout::Name;
use newtype::Counter;
use split::Split;

#[derive(Clone, Debug, PartialEq)]
pub struct Cut {
    conductance: Conductance,
    inside: Vec<Name>,
    outside: Vec<Name>,
}

impl Cut {
    pub fn best(graph: &ItemGraph, tangles: &[Vec<NodeIndex>], floor: Least) -> Option<Cut> {
        let whole = Condensed::of_tangles(graph, tangles);
        let islands = whole.islands();
        let sweeps =
            islands.iter().filter(|island| island.len() > 1).filter_map(|island| whole.restricted(island).sweep(floor));
        Self::apart(&whole, &islands, floor)
            .into_iter()
            .chain(sweeps)
            .min_by(|left, right| left.conductance().total_cmp(right.conductance()))
            .map(|split| Cut {
                conductance: split.conductance(),
                inside: Self::names(graph, tangles, split.inside()),
                outside: Self::names(graph, tangles, &split.into_outside()),
            })
    }

    pub fn conductance(&self) -> Conductance {
        self.conductance
    }

    fn apart(whole: &Condensed, islands: &[BTreeSet<TangleIndex>], floor: Least) -> Option<Split> {
        let total: BTreeSet<TangleIndex> = islands.iter().flatten().copied().collect();
        let inside = Self::balanced(whole, islands, &total);
        let outside: BTreeSet<TangleIndex> = total.difference(&inside).copied().collect();
        let smaller = whole.items(&inside).count().min(whole.items(&outside).count());
        floor.allows(smaller).then(|| Split::new(whole.conductance(&inside), inside, outside))
    }

    fn balanced(
        condensed: &Condensed,
        islands: &[BTreeSet<TangleIndex>],
        total: &BTreeSet<TangleIndex>,
    ) -> BTreeSet<TangleIndex> {
        let mut islands = islands.to_vec();
        islands.sort_by_key(|island| std::cmp::Reverse(condensed.items(island)));
        let whole = condensed.items(total);
        islands.into_iter().fold(BTreeSet::new(), |inside, island| {
            if condensed.items(&inside).plus(condensed.items(&inside)) < whole {
                inside.into_iter().chain(island).collect()
            } else {
                inside
            }
        })
    }

    fn names(graph: &ItemGraph, tangles: &[Vec<NodeIndex>], side: &BTreeSet<TangleIndex>) -> Vec<Name> {
        side.iter()
            .filter_map(|tangle| tangles.get(tangle.index()))
            .flatten()
            .filter_map(|member| graph.name_at(*member))
            .cloned()
            .collect()
    }
}

impl fmt::Display for Cut {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list = |names: &[Name]| names.iter().map(ToString::to_string).collect::<Vec<String>>().join(" ");
        write!(
            formatter,
            "conductance {} between {} items [{}] and {} items [{}]",
            self.conductance,
            self.inside.len(),
            list(&self.inside),
            self.outside.len(),
            list(&self.outside)
        )
    }
}
