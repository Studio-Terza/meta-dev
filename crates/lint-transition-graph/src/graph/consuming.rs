use std::collections::{BTreeMap, BTreeSet};

mod spread;

use super::Graph;
use super::hyperedge::Hyperedge;
use super::label::Label;
use crate::end::{Hop, Node, Tuple};
use spread::Spread;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Moves {
    All,
    Unlifted,
}

impl Moves {
    fn kept<'edge, 'graph, 'file>(
        self,
        edge: &'edge Hyperedge<'graph, 'file>,
    ) -> Option<&'edge Hyperedge<'graph, 'file>> {
        match self {
            Moves::All => Some(edge),
            Moves::Unlifted => AsRef::<Label>::as_ref(edge).lifted().map_or(Some(edge), |_| None),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Consuming<'graph, 'file> {
    by: BTreeMap<&'graph Node<'graph>, Vec<&'graph Hyperedge<'graph, 'file>>>,
}

impl<'graph, 'file> Consuming<'graph, 'file> {
    pub(crate) fn indexed(graph: &'graph Graph<'_, 'file>, moves: Moves) -> Consuming<'graph, 'file> {
        let kept = graph.hyperedges().iter().filter_map(|edge| moves.kept(edge));
        let by = kept.fold(
            BTreeMap::new(),
            |mut by: BTreeMap<&'graph Node<'graph>, Vec<&'graph Hyperedge<'graph, 'file>>>, edge| {
                let hop: &'graph Hop<'graph> = edge.as_ref();
                AsRef::<Tuple>::as_ref(hop).iter().for_each(|node| by.entry(node).or_default().push(edge));
                by
            },
        );
        Consuming { by }
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = &'graph Node<'graph>> + '_ {
        self.by.keys().copied()
    }

    pub(crate) fn taking(&self, node: &Node<'graph>) -> impl Iterator<Item = &'graph Hyperedge<'graph, 'file>> + '_ {
        self.by.get(node).into_iter().flatten().copied()
    }

    pub(crate) fn led_within<'query>(
        &'query self,
        node: &'query Node<'graph>,
        members: &'query BTreeSet<&'graph Node<'graph>>,
    ) -> impl Iterator<Item = &'graph Hyperedge<'graph, 'file>> + 'query {
        self.taking(node).filter(move |edge| {
            AsRef::<Tuple>::as_ref(AsRef::<Hop>::as_ref(*edge)).iter().find(|taken| members.contains(taken))
                == Some(node)
        })
    }

    pub(crate) fn closure(&self, start: &BTreeSet<&'graph Node<'graph>>) -> BTreeSet<&'graph Node<'graph>> {
        self.grown(BTreeSet::new(), start)
    }

    fn grown(
        &self,
        mut reached: BTreeSet<&'graph Node<'graph>>,
        frontier: &BTreeSet<&'graph Node<'graph>>,
    ) -> BTreeSet<&'graph Node<'graph>> {
        reached.extend(frontier.iter().copied());
        let (_, fresh): (BTreeSet<&'graph Node<'graph>>, BTreeSet<&'graph Node<'graph>>) = frontier
            .iter()
            .flat_map(|node| self.taking(node))
            .filter_map(|edge| AsRef::<Hop>::as_ref(edge).fired(&reached))
            .partition(|node| reached.contains(node));
        Spread::new(reached, fresh).grown(self)
    }
}
