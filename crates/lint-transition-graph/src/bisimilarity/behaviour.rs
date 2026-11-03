use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::iter;

use newtype::{Arbitrary, Display, Newtype};

use crate::count::Count;
use crate::edge::Edge;
use crate::end::{End, Node};
use crate::graph::{Graph, Interface, Label};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Block(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Destination<'graph> {
    Domain(&'graph Node<'graph>),
    Outside(&'graph End<'graph>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Move<'graph> {
    label: Label<'graph>,
    destination: Destination<'graph>,
    with: BTreeSet<&'graph Node<'graph>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reached<'graph> {
    Block(Block),
    Outside(&'graph End<'graph>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Seen<'graph> {
    label: &'graph Label<'graph>,
    reached: Reached<'graph>,
    with: BTreeSet<Block>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Class<'graph> {
    block: Block,
    seen: BTreeSet<Seen<'graph>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Filler<'graph> {
    interface: Interface<'graph>,
    node: &'graph Node<'graph>,
}

#[derive(Debug)]
pub(super) struct SetAside {
    interfaces: Count,
    fillers: Count,
}

impl fmt::Display for SetAside {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} interface nodes with {} fillers", self.interfaces, self.fillers)
    }
}

#[derive(Debug)]
struct Labelled<'graph> {
    node: &'graph Node<'graph>,
    found: Move<'graph>,
}

impl<'graph> Labelled<'graph> {
    fn of_edge(edge: &'graph Edge<'_, '_>) -> Vec<Labelled<'graph>> {
        let label = Label::read(edge.role(), edge.transition());
        let gives: &'graph End<'graph> = edge.ends().as_ref();
        let destination = gives.node().map_or(Destination::Outside(gives), Destination::Domain);
        let taken: BTreeSet<&'graph Node<'graph>> = edge.ends().taken().collect();
        taken
            .iter()
            .map(|node| Labelled {
                node,
                found: Move { label, destination, with: taken.iter().copied().filter(|other| other != node).collect() },
            })
            .collect()
    }

    fn filler(&self) -> Option<Filler<'graph>> {
        self.found.label.interface().map(|interface| Filler { interface, node: self.node })
    }
}

#[derive(Debug)]
pub(super) struct Behaviour<'graph> {
    moves: BTreeMap<&'graph Node<'graph>, BTreeSet<Move<'graph>>>,
    fillers: BTreeSet<Filler<'graph>>,
}

impl<'graph> Behaviour<'graph> {
    pub(super) fn read(graph: &'graph Graph<'_, '_>) -> Behaviour<'graph> {
        let start: BTreeMap<&'graph Node<'graph>, BTreeSet<Move<'graph>>> =
            graph.nodes().keys().map(|node| (node, BTreeSet::new())).collect();
        let labelled: Vec<Labelled<'graph>> = graph.edges().iter().flat_map(Labelled::of_edge).collect();
        let fillers = labelled.iter().filter_map(Labelled::filler).collect();
        let moves = labelled.into_iter().filter(|found| found.filler().is_none()).fold(start, |mut moves, found| {
            moves.entry(found.node).or_default().insert(found.found);
            moves
        });
        Behaviour { moves, fillers }
    }

    pub(super) fn set_aside(&self) -> SetAside {
        SetAside {
            interfaces: Count::new(
                self.fillers.iter().map(|filler| &filler.interface).collect::<BTreeSet<&Interface>>().len(),
            ),
            fillers: Count::new(self.fillers.len()),
        }
    }

    pub(super) fn refined(&'graph self) -> BTreeMap<&'graph Node<'graph>, Block> {
        let start: BTreeMap<&'graph Node<'graph>, Block> =
            self.moves.keys().map(|node| (*node, Block::default())).collect();
        iter::successors(Some(start), |blocks| {
            let next = self.split(blocks);
            (Self::distinct(&next) != Self::distinct(blocks)).then_some(next)
        })
        .last()
        .unwrap_or_default()
    }

    pub(super) fn moves_of(&self, node: &Node<'graph>) -> Option<&BTreeSet<Move<'graph>>> {
        self.moves.get(node).filter(|moves| !moves.is_empty())
    }

    fn split(&'graph self, blocks: &BTreeMap<&'graph Node<'graph>, Block>) -> BTreeMap<&'graph Node<'graph>, Block> {
        let classes: BTreeMap<&'graph Node<'graph>, Class<'graph>> = self
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

    fn distinct(blocks: &BTreeMap<&'graph Node<'graph>, Block>) -> Count {
        Count::new(blocks.values().collect::<BTreeSet<&Block>>().len())
    }
}

impl<'graph> Move<'graph> {
    fn seen(&'graph self, blocks: &BTreeMap<&'graph Node<'graph>, Block>) -> Seen<'graph> {
        let reached = match self.destination {
            Destination::Domain(node) => Reached::Block(blocks.get(node).copied().unwrap_or_default()),
            Destination::Outside(end) => Reached::Outside(end),
        };
        let with = self.with.iter().map(|node| blocks.get(*node).copied().unwrap_or_default()).collect();
        Seen { label: &self.label, reached, with }
    }
}

impl fmt::Display for Move<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.destination {
            Destination::Domain(node) => write!(formatter, "{} to {node}", self.label),
            Destination::Outside(end) => write!(formatter, "{} to {end}", self.label),
        }?;
        let with: Vec<String> = self.with.iter().map(ToString::to_string).collect();
        match with.as_slice() {
            [] => Ok(()),
            _ => write!(formatter, " with {}", with.join(", ")),
        }
    }
}
