use algebra::{Catenation, Monoid, Semigroup};
use newtype::{Arbitrary, Counter, Newtype};
use syntax::Node;

use crate::watch::Kind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Kinds(u128);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Offset(usize);

#[derive(Clone, Copy, Debug)]
struct Subtree {
    descendants: Offset,
    kinds: Kinds,
}

#[derive(Debug, Default)]
pub(crate) struct KindIndex(Vec<Subtree>);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Indexed<'index> {
    from: &'index [Subtree],
}

#[derive(Clone, Debug)]
pub(crate) struct Cursors<'index> {
    rest: &'index [Subtree],
}

#[derive(Debug, Default)]
struct Built {
    subtrees: Catenation<Subtree>,
    size: Offset,
    kinds: Kinds,
}

impl Kinds {
    pub(crate) fn listed(kinds: &[Kind]) -> Kinds {
        Monoid::concat(kinds.iter().copied().map(Kinds::from))
    }

    fn overlap(self, other: Kinds) -> Option<()> {
        (self.0 & other.0 != 0).then_some(())
    }
}

impl From<Kind> for Kinds {
    fn from(kind: Kind) -> Kinds {
        let bit = Kind::ALL.iter().position(|listed| *listed == kind).and_then(|at| u32::try_from(at).ok());
        Kinds(bit.and_then(|at| 1_u128.checked_shl(at)).unwrap_or_default())
    }
}

impl Semigroup for Kinds {
    fn combine(self, other: Kinds) -> Kinds {
        Kinds(self.0 | other.0)
    }
}

impl Monoid for Kinds {}

impl Semigroup for Built {
    fn combine(self, other: Built) -> Built {
        Built {
            subtrees: self.subtrees.combine(other.subtrees),
            size: self.size.plus(other.size),
            kinds: self.kinds.combine(other.kinds),
        }
    }
}

impl Monoid for Built {}

impl From<Node<'_>> for Built {
    fn from(node: Node<'_>) -> Built {
        let below: Built = Monoid::concat(node.children().into_iter().map(Built::from));
        let own = Subtree { descendants: below.size, kinds: Kinds::from(Kind::of(node)).combine(below.kinds) };
        Built {
            subtrees: Catenation::from(vec![own]).combine(below.subtrees),
            size: below.size.incremented(),
            kinds: own.kinds,
        }
    }
}

impl From<&syntax::File> for KindIndex {
    fn from(file: &syntax::File) -> KindIndex {
        let built: Built = Monoid::concat(Node::top_level(file).into_iter().map(Built::from));
        KindIndex(built.subtrees.into_iter().collect())
    }
}

impl KindIndex {
    pub(crate) fn top(&self) -> Cursors<'_> {
        Cursors { rest: &self.0 }
    }
}

impl<'index> Indexed<'index> {
    pub(crate) fn watches(self, kinds: Kinds) -> Option<()> {
        self.from.first()?.kinds.overlap(kinds)
    }

    pub(crate) fn children(self) -> Cursors<'index> {
        let below = self.from.split_first().and_then(|(here, below)| below.get(..here.descendants.count()));
        Cursors { rest: below.unwrap_or_default() }
    }
}

impl<'index> Iterator for Cursors<'index> {
    type Item = Indexed<'index>;

    fn next(&mut self) -> Option<Indexed<'index>> {
        let from = self.rest;
        let (here, below) = from.split_first()?;
        self.rest = below.get(here.descendants.count()..).unwrap_or_default();
        Some(Indexed { from })
    }
}
