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
    size: Offset,
    kinds: Kinds,
}

#[derive(Debug, Default)]
pub(crate) struct KindIndex(Vec<Subtree>);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Indexed<'index> {
    index: &'index KindIndex,
    at: Offset,
}

#[derive(Clone, Debug)]
pub(crate) struct Cursors<'index> {
    index: &'index KindIndex,
    at: Offset,
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
        let own = Subtree { size: below.size.incremented(), kinds: Kinds::from(Kind::of(node)).combine(below.kinds) };
        Built { subtrees: Catenation::from(vec![own]).combine(below.subtrees), size: own.size, kinds: own.kinds }
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
        Cursors { index: self, at: Offset::zero(), rest: &self.0 }
    }

    fn subtree(&self, at: Offset) -> Option<&Subtree> {
        self.0.get(at.count())
    }
}

impl<'index> Indexed<'index> {
    pub(crate) fn watches(self, kinds: Kinds) -> Option<()> {
        self.index.subtree(self.at)?.kinds.overlap(kinds)
    }

    pub(crate) fn children(self) -> Cursors<'index> {
        let start = self.at.incremented();
        let size = self.index.subtree(self.at).map_or(Offset::zero().incremented(), |here| here.size);
        let rest = self.index.0.get(start.count()..self.at.plus(size).count()).unwrap_or_default();
        Cursors { index: self.index, at: start, rest }
    }
}

impl<'index> Iterator for Cursors<'index> {
    type Item = Indexed<'index>;

    fn next(&mut self) -> Option<Indexed<'index>> {
        let (first, _) = self.rest.split_first()?;
        let here = self.at;
        self.at = here.plus(first.size);
        self.rest = self.rest.get(first.size.count()..).unwrap_or_default();
        Some(Indexed { index: self.index, at: here })
    }
}
