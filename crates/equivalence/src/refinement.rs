use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use algebra::{Act, Monoid};
use newtype::{Arbitrary, Display, Newtype};

use crate::cluster::{Cluster, Placed};
use crate::erased::Laws;
use crate::renaming::{Pair, Renaming};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Colour(usize);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
struct Colours(usize);

impl Colours {
    fn counted<Node>(colours: &BTreeMap<Node, Colour>) -> Colours {
        Colours::new(colours.values().collect::<BTreeSet<&Colour>>().len())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move<Node, Label, Law> {
    label: Label,
    reached: Option<Node>,
    with: BTreeSet<Node>,
    laws: Vec<Law>,
}

impl<Node: Ord + Copy, Label: Ord, Law: Ord> Move<Node, Label, Law> {
    pub fn new(label: Label, reached: Option<Node>, with: BTreeSet<Node>, laws: Vec<Law>) -> Move<Node, Label, Law> {
        Move { label, reached, with, laws }
    }

    fn seen(&self, colours: &BTreeMap<Node, Colour>) -> Seen<'_, Label> {
        Seen {
            label: &self.label,
            reached: self.reached.map(|node| colours.get(&node).copied().unwrap_or_default()),
            with: self.with.iter().map(|node| colours.get(node).copied().unwrap_or_default()).collect(),
        }
    }

    fn paired(&self, ours: &Move<Node, Label, Law>, lens: Lens<'_, Node>) -> Option<Vec<Pair<Node>>> {
        let agreed = lens.agreeing(&self.laws, &ours.laws).filter(|_| self.label == ours.label)?;
        let with = agreed.ordered(&self.with).into_iter().zip(agreed.ordered(&ours.with));
        Some(
            iter::once(Pair::new(self.reached, ours.reached))
                .chain(with.map(|(from, to)| Pair::new(Some(from), Some(to))))
                .collect(),
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct Lens<'colours, Node> {
    colours: &'colours BTreeMap<Node, Colour>,
    laws: Option<Laws>,
}

#[derive(Clone, Copy, Debug)]
struct Agreed<'colours, Node> {
    colours: &'colours BTreeMap<Node, Colour>,
}

impl<'colours, Node: Ord + Copy> Lens<'colours, Node> {
    fn agreeing<Law: PartialEq>(self, ours: &[Law], theirs: &[Law]) -> Option<Agreed<'colours, Node>> {
        let agreed = Agreed { colours: self.colours };
        self.laws.map_or(Some(agreed), |laws| laws.agree(ours, theirs).map(|_| agreed))
    }
}

impl<Node: Ord + Copy> Agreed<'_, Node> {
    fn ordered(self, nodes: &BTreeSet<Node>) -> Vec<Node> {
        let mut ordered: Vec<Node> = nodes.iter().copied().collect();
        ordered.sort_by_key(|node| self.colours.get(node).copied());
        ordered
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Seen<'moves, Label> {
    label: &'moves Label,
    reached: Option<Colour>,
    with: Vec<Colour>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Class<'moves, Label> {
    colour: Colour,
    seen: Vec<Seen<'moves, Label>>,
}

#[derive(Debug)]
pub struct Refinement<Node, Label, Law> {
    moves: BTreeMap<Node, Vec<Move<Node, Label, Law>>>,
    laws: Option<Laws>,
}

impl<Node: Ord + Copy, Label: Ord, Law: Ord> Refinement<Node, Label, Law> {
    #[must_use]
    pub fn new(moves: BTreeMap<Node, Vec<Move<Node, Label, Law>>>, laws: Option<Laws>) -> Refinement<Node, Label, Law> {
        Refinement { moves, laws }
    }

    #[must_use]
    pub fn coloured(&self) -> BTreeMap<Node, Colour> {
        let start: BTreeMap<Node, Colour> = self.moves.keys().map(|node| (*node, Colour::default())).collect();
        iter::successors(Some(start), |colours| {
            let next = self.split(colours);
            (Colours::counted(&next) != Colours::counted(colours)).then_some(next)
        })
        .last()
        .unwrap_or_default()
    }

    #[must_use]
    pub fn moves_of(&self, node: &Node) -> &[Move<Node, Label, Law>] {
        self.moves.get(node).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn clustered(
        &self,
        colours: &BTreeMap<Node, Colour>,
        alike: impl Fn(Node, Node) -> Option<Node>,
    ) -> Vec<Cluster<Node>> {
        let coloured =
            colours.iter().fold(BTreeMap::new(), |mut found: BTreeMap<Colour, Vec<Node>>, (node, colour)| {
                found.entry(*colour).or_default().push(*node);
                found
            });
        coloured
            .values()
            .flat_map(|nodes| {
                let placed = nodes.iter().fold(Vec::new(), |mut placed, node| {
                    let placing = self.placing(&placed, *node, colours, &alike);
                    placed.push(placing);
                    placed
                });
                Placed::gathered(placed)
            })
            .filter(|cluster| cluster.members().nth(1).is_some())
            .collect()
    }

    fn placing(
        &self,
        placed: &[Placed<Node>],
        node: Node,
        colours: &BTreeMap<Node, Colour>,
        alike: &impl Fn(Node, Node) -> Option<Node>,
    ) -> Placed<Node> {
        placed
            .iter()
            .filter_map(Placed::leader)
            .find_map(|leader| {
                self.renaming(leader, node, colours, alike).map(|renaming| Placed::Joins { leader, renaming })
            })
            .unwrap_or(Placed::Leads(node))
    }

    fn renaming(
        &self,
        first: Node,
        other: Node,
        colours: &BTreeMap<Node, Colour>,
        alike: &impl Fn(Node, Node) -> Option<Node>,
    ) -> Option<Renaming<Node>> {
        alike(first, other)?;
        let ours = self.aligned(first, colours);
        let theirs = self.aligned(other, colours);
        let lens = Lens { colours, laws: self.laws };
        let aligned = Some(theirs.len()).filter(|count| *count == ours.len());
        let pairs = aligned.and_then(|_| {
            theirs
                .iter()
                .zip(&ours)
                .map(|(their, our)| their.paired(our, lens))
                .collect::<Option<Vec<Vec<Pair<Node>>>>>()
        })?;
        Renaming::started(other, first).acted(Monoid::concat(pairs))
    }

    fn aligned(&self, node: Node, colours: &BTreeMap<Node, Colour>) -> Vec<&Move<Node, Label, Law>> {
        let mut moves: Vec<&Move<Node, Label, Law>> = self.moves_of(&node).iter().collect();
        moves.sort_by(|left, right| {
            left.seen(colours).cmp(&right.seen(colours)).then_with(|| left.laws.cmp(&right.laws))
        });
        moves
    }

    fn split(&self, colours: &BTreeMap<Node, Colour>) -> BTreeMap<Node, Colour> {
        let classes: BTreeMap<Node, Class<'_, Label>> = self
            .moves
            .iter()
            .map(|(node, moves)| {
                let colour = colours.get(node).copied().unwrap_or_default();
                let mut seen: Vec<Seen<'_, Label>> = moves.iter().map(|found| found.seen(colours)).collect();
                seen.sort();
                (*node, Class { colour, seen })
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
}
