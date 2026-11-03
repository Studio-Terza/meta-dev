mod condensed;
mod conductance;
mod split;

use std::collections::BTreeSet;
use std::fmt;

use crate::crate_summary::cut::conductance::Conductance;
use crate::crate_summary::item_count::ItemCount;
use condensed::{Condensed, TangleIndex};
use lint_core::Bound;
use lint_core::Fix;
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
    pub fn best(graph: &ItemGraph, tangles: &[Vec<NodeIndex>], floor: Bound) -> Option<Cut> {
        let whole = Condensed::of_tangles(graph, tangles);
        let islands = whole.islands();
        let sweeps = islands.iter().filter_map(|island| whole.restricted(island).sweep(floor));
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

    fn apart(whole: &Condensed, islands: &[BTreeSet<TangleIndex>], floor: Bound) -> Option<Split> {
        let total: BTreeSet<TangleIndex> = islands.iter().flatten().copied().collect();
        let inside = Self::balanced(whole, islands, &total);
        let outside: BTreeSet<TangleIndex> = total.difference(&inside).copied().collect();
        let inside_items = ItemCount::total(inside.iter().map(|tangle| whole.size(*tangle)));
        let outside_items = ItemCount::total(outside.iter().map(|tangle| whole.size(*tangle)));
        let smaller = inside_items.min(outside_items);
        floor.allows(smaller).map(|_| Split::new(whole.conductance(&inside), inside, outside))
    }

    fn balanced(
        condensed: &Condensed,
        islands: &[BTreeSet<TangleIndex>],
        total: &BTreeSet<TangleIndex>,
    ) -> BTreeSet<TangleIndex> {
        let mut islands = islands.to_vec();
        islands.sort_by_key(|island| {
            std::cmp::Reverse(ItemCount::total(island.iter().map(|tangle| condensed.size(*tangle))))
        });
        let whole = ItemCount::total(total.iter().map(|tangle| condensed.size(*tangle)));
        islands.into_iter().fold(BTreeSet::new(), |inside, island| {
            let held = ItemCount::total(inside.iter().map(|tangle| condensed.size(*tangle)));
            if held.plus(held) < whole { inside.into_iter().chain(island).collect() } else { inside }
        })
    }

    fn names(graph: &ItemGraph, tangles: &[Vec<NodeIndex>], side: &BTreeSet<TangleIndex>) -> Vec<Name> {
        side.iter()
            .filter_map(|tangle| tangles.get(tangle.into_inner()))
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

impl lint_core::Finding for Cut {
    fn fix(&self) -> Fix {
        Fix::new("move the two groups into crates of their own")
    }
}
