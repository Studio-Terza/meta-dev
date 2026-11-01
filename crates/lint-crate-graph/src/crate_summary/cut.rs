mod condensed;
mod conductance;

use std::collections::BTreeSet;
use std::fmt;

use crate::crate_summary::cut::conductance::Conductance;
use crate::crate_summary::item_count::ItemCount;
use crate::item_graph::{ItemGraph, NodeIndex};
use condensed::{Condensed, TangleIndex};
use lint_core::Name;

#[derive(Clone, Debug, PartialEq)]
pub struct Cut {
    conductance: Conductance,
    inside: Vec<Name>,
    outside: Vec<Name>,
}

impl Cut {
    pub fn best(graph: &ItemGraph, tangles: &[Vec<NodeIndex>]) -> Option<Cut> {
        let condensed = Condensed::of_tangles(graph, tangles);
        Self::best_side(&condensed).map(|side| Cut {
            conductance: condensed.conductance(&side),
            inside: Self::names(graph, tangles, &side),
            outside: Self::names(graph, tangles, &condensed.all().difference(&side).copied().collect()),
        })
    }

    pub fn conductance(&self) -> Conductance {
        self.conductance
    }

    pub fn smaller_side(&self) -> ItemCount {
        ItemCount::of_len(self.inside.len().min(self.outside.len()))
    }

    fn best_side(condensed: &Condensed) -> Option<BTreeSet<TangleIndex>> {
        let islands = condensed.islands();
        if islands.len() > 1 {
            Some(Self::balanced(condensed, islands))
        } else {
            let order = condensed.sweep_order();
            (1..order.len())
                .map(|prefix| order.iter().take(prefix).copied().collect::<BTreeSet<TangleIndex>>())
                .min_by(|left, right| condensed.conductance(left).total_cmp(condensed.conductance(right)))
        }
    }

    fn balanced(condensed: &Condensed, islands: Vec<BTreeSet<TangleIndex>>) -> BTreeSet<TangleIndex> {
        let mut islands = islands;
        islands.sort_by_key(|island| std::cmp::Reverse(condensed.items(island)));
        let total = condensed.items(&condensed.all());
        islands.into_iter().fold(BTreeSet::new(), |inside, island| {
            if condensed.items(&inside) + condensed.items(&inside) < total {
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
