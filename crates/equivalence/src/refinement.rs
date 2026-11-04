use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use newtype::{Arbitrary, Display, Newtype};

use crate::renaming::Renaming;

pub trait Agreeing {
    fn agrees<'other>(&self, other: &'other Self) -> Option<&'other Self>;
}

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Colour(usize);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
struct Colours(usize);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move<Node, Label> {
    label: Label,
    reached: Option<Node>,
    with: BTreeSet<Node>,
}

impl<Node: Ord + Copy, Label: Ord> Move<Node, Label> {
    pub fn new(label: Label, reached: Option<Node>, with: BTreeSet<Node>) -> Move<Node, Label> {
        Move { label, reached, with }
    }

    fn seen(&self, colours: &BTreeMap<Node, Colour>) -> Seen<'_, Label> {
        Seen {
            label: &self.label,
            reached: self.reached.map(|node| colours.get(&node).copied().unwrap_or_default()),
            with: self.with.iter().map(|node| colours.get(node).copied().unwrap_or_default()).collect(),
        }
    }
}

impl<Node: Ord + Copy, Label: Ord + Agreeing> Move<Node, Label> {
    fn renamed(
        &self,
        ours: &Move<Node, Label>,
        renaming: Renaming<Node>,
        colours: &BTreeMap<Node, Colour>,
    ) -> Option<Renaming<Node>> {
        let mut theirs: Vec<Node> = self.with.iter().copied().collect();
        let mut mine: Vec<Node> = ours.with.iter().copied().collect();
        theirs.sort_by_key(|node| colours.get(node).copied());
        mine.sort_by_key(|node| colours.get(node).copied());
        theirs
            .into_iter()
            .zip(mine)
            .try_fold(renaming.through(self.reached, ours.reached)?, |renaming, (from, to)| renaming.paired(from, to))
            .filter(|_| self.label.agrees(&ours.label).is_some())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Seen<'moves, Label> {
    label: &'moves Label,
    reached: Option<Colour>,
    with: BTreeSet<Colour>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Class<'moves, Label> {
    colour: Colour,
    seen: BTreeSet<Seen<'moves, Label>>,
}

#[derive(Debug)]
pub struct Refinement<Node, Label> {
    moves: BTreeMap<Node, BTreeSet<Move<Node, Label>>>,
}

impl<Node: Ord + Copy, Label: Ord> Refinement<Node, Label> {
    #[must_use]
    pub fn new(moves: BTreeMap<Node, BTreeSet<Move<Node, Label>>>) -> Refinement<Node, Label> {
        Refinement { moves }
    }

    #[must_use]
    pub fn coloured(&self) -> BTreeMap<Node, Colour> {
        let start: BTreeMap<Node, Colour> = self.moves.keys().map(|node| (*node, Colour::default())).collect();
        iter::successors(Some(start), |colours| {
            let next = self.split(colours);
            (Self::distinct(&next) != Self::distinct(colours)).then_some(next)
        })
        .last()
        .unwrap_or_default()
    }

    #[must_use]
    pub fn moves_of(&self, node: &Node) -> Option<&BTreeSet<Move<Node, Label>>> {
        self.moves.get(node)
    }

    fn split(&self, colours: &BTreeMap<Node, Colour>) -> BTreeMap<Node, Colour> {
        let classes: BTreeMap<Node, Class<'_, Label>> = self
            .moves
            .iter()
            .map(|(node, moves)| {
                let colour = colours.get(node).copied().unwrap_or_default();
                (*node, Class { colour, seen: moves.iter().map(|found| found.seen(colours)).collect() })
            })
            .collect();
        let numbered: BTreeMap<&Class<'_, Label>, Colour> = classes
            .values()
            .collect::<BTreeSet<&Class<'_, Label>>>()
            .into_iter()
            .enumerate()
            .map(|(index, class)| (class, Colour::new(index)))
            .collect();
        classes.iter().map(|(node, class)| (*node, numbered.get(class).copied().unwrap_or_default())).collect()
    }

    fn distinct(colours: &BTreeMap<Node, Colour>) -> Colours {
        Colours::new(colours.values().collect::<BTreeSet<&Colour>>().len())
    }
}

impl<Node: Ord + Copy, Label: Ord + Agreeing> Refinement<Node, Label> {
    #[must_use]
    pub fn renaming(&self, first: Node, other: Node, colours: &BTreeMap<Node, Colour>) -> Option<Renaming<Node>> {
        let ours = self.aligned(first, colours);
        let theirs = self.aligned(other, colours);
        Some(theirs.len()).filter(|count| *count == ours.len()).and_then(|_| {
            theirs.iter().zip(&ours).try_fold(Renaming::started(other, first), |renaming, (their, our)| {
                their.renamed(our, renaming, colours)
            })
        })
    }

    fn aligned(&self, node: Node, colours: &BTreeMap<Node, Colour>) -> Vec<&Move<Node, Label>> {
        let mut moves: Vec<&Move<Node, Label>> = self.moves.get(&node).into_iter().flatten().collect();
        moves.sort_by_key(|found| found.seen(colours));
        moves
    }
}
