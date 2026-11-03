use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::iter;

use newtype::{Arbitrary, Display, Newtype};

use crate::count::Count;
use crate::end::{End, Node};
use crate::graph::{Graph, Label};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Block(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Destination<'graph> {
    Domain(&'graph Node),
    Outside(&'graph End),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Move<'graph> {
    label: Label,
    destination: Destination<'graph>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reached<'graph> {
    Block(Block),
    Outside(&'graph End),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Seen<'graph> {
    label: &'graph Label,
    reached: Reached<'graph>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Class<'graph> {
    block: Block,
    seen: BTreeSet<Seen<'graph>>,
}

#[derive(Debug)]
pub(super) struct Behaviour<'graph> {
    moves: BTreeMap<&'graph Node, BTreeSet<Move<'graph>>>,
}

impl<'graph> Behaviour<'graph> {
    pub(super) fn read(graph: &'graph Graph<'_>) -> Behaviour<'graph> {
        let start: BTreeMap<&'graph Node, BTreeSet<Move<'graph>>> =
            graph.nodes().keys().map(|node| (node, BTreeSet::new())).collect();
        let moves = graph
            .edges()
            .iter()
            .flat_map(|edge| {
                let label = Label::read(edge.role(), edge.transition());
                let gives: &'graph End = edge.ends().as_ref();
                let destination = gives.node().map_or(Destination::Outside(gives), Destination::Domain);
                edge.ends()
                    .taken()
                    .map(|node| (node, Move { label: label.clone(), destination }))
                    .collect::<Vec<(&'graph Node, Move<'graph>)>>()
            })
            .fold(start, |mut moves, (node, found)| {
                moves.entry(node).or_default().insert(found);
                moves
            });
        Behaviour { moves }
    }

    pub(super) fn refined(&'graph self) -> BTreeMap<&'graph Node, Block> {
        let start: BTreeMap<&'graph Node, Block> = self.moves.keys().map(|node| (*node, Block::default())).collect();
        iter::successors(Some(start), |blocks| {
            let next = self.split(blocks);
            (Self::distinct(&next) != Self::distinct(blocks)).then_some(next)
        })
        .last()
        .unwrap_or_default()
    }

    pub(super) fn moves_of(&self, node: &Node) -> Option<&BTreeSet<Move<'graph>>> {
        self.moves.get(node).filter(|moves| !moves.is_empty())
    }

    fn split(&'graph self, blocks: &BTreeMap<&'graph Node, Block>) -> BTreeMap<&'graph Node, Block> {
        let classes: BTreeMap<&'graph Node, Class<'graph>> = self
            .moves
            .iter()
            .map(|(node, moves)| {
                let block = blocks.get(node).copied().unwrap_or_default();
                let seen = moves.iter().map(|found| found.seen(blocks)).collect();
                (*node, Class { block, seen })
            })
            .collect();
        let numbered: BTreeMap<&Class<'graph>, Block> = classes
            .values()
            .collect::<BTreeSet<&Class<'graph>>>()
            .into_iter()
            .enumerate()
            .map(|(index, class)| (class, Block::new(index)))
            .collect();
        classes.iter().map(|(node, class)| (*node, numbered.get(class).copied().unwrap_or_default())).collect()
    }

    fn distinct(blocks: &BTreeMap<&'graph Node, Block>) -> Count {
        Count::new(blocks.values().collect::<BTreeSet<&Block>>().len())
    }
}

impl<'graph> Move<'graph> {
    fn seen(&'graph self, blocks: &BTreeMap<&'graph Node, Block>) -> Seen<'graph> {
        let reached = match self.destination {
            Destination::Domain(node) => Reached::Block(blocks.get(node).copied().unwrap_or_default()),
            Destination::Outside(end) => Reached::Outside(end),
        };
        Seen { label: &self.label, reached }
    }
}

impl fmt::Display for Move<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.destination {
            Destination::Domain(node) => write!(formatter, "{} to {node}", self.label),
            Destination::Outside(end) => write!(formatter, "{} to {end}", self.label),
        }
    }
}
