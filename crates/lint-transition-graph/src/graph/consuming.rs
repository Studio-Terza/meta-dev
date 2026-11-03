use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use super::Graph;
use super::hyperedge::Hyperedge;
use super::label::Label;
use crate::end::{Hop, Node, Tuple};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Moves {
    All,
    Unlifted,
}

#[derive(Debug)]
pub(crate) struct Consuming<'graph, 'found> {
    by: BTreeMap<&'graph Node, Vec<&'graph Hyperedge<'found>>>,
}

impl<'graph, 'found> Consuming<'graph, 'found> {
    pub(crate) fn indexed(graph: &'graph Graph<'found>, moves: Moves) -> Consuming<'graph, 'found> {
        let kept = graph.hyperedges().iter().filter(|edge| match moves {
            Moves::All => true,
            Moves::Unlifted => AsRef::<Label>::as_ref(*edge).lifted().is_none(),
        });
        let by = kept.fold(BTreeMap::new(), |mut by: BTreeMap<&'graph Node, Vec<&'graph Hyperedge<'found>>>, edge| {
            let hop: &'graph Hop = edge.as_ref();
            AsRef::<Tuple>::as_ref(hop).iter().for_each(|node| by.entry(node).or_default().push(edge));
            by
        });
        Consuming { by }
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = &'graph Node> + '_ {
        self.by.keys().copied()
    }

    pub(crate) fn taking(&self, node: &Node) -> impl Iterator<Item = &'graph Hyperedge<'found>> + '_ {
        self.by.get(node).into_iter().flatten().copied()
    }

    pub(crate) fn led<'query>(
        &'query self,
        node: &'query Node,
    ) -> impl Iterator<Item = &'graph Hyperedge<'found>> + 'query {
        self.taking(node).filter(move |edge| AsRef::<Tuple>::as_ref(AsRef::<Hop>::as_ref(*edge)).first() == Some(node))
    }

    pub(crate) fn led_within<'query>(
        &'query self,
        node: &'query Node,
        members: &'query BTreeSet<&'graph Node>,
    ) -> impl Iterator<Item = &'graph Hyperedge<'found>> + 'query {
        self.taking(node).filter(move |edge| {
            AsRef::<Tuple>::as_ref(AsRef::<Hop>::as_ref(*edge)).iter().find(|taken| members.contains(taken))
                == Some(node)
        })
    }

    pub(crate) fn closure(&self, start: BTreeSet<&'graph Node>) -> BTreeSet<&'graph Node> {
        iter::successors(Some((start.clone(), start)), |(reached, frontier)| {
            let fresh: BTreeSet<&'graph Node> = frontier
                .iter()
                .flat_map(|node| self.taking(node))
                .filter_map(|edge| AsRef::<Hop>::as_ref(edge).fired(reached))
                .filter(|node| !reached.contains(node))
                .collect();
            fresh.first().map(|_| (reached.union(&fresh).copied().collect(), fresh.clone()))
        })
        .last()
        .map_or_else(BTreeSet::new, |(reached, _)| reached)
    }
}
