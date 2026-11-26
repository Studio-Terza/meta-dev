use algebra::{Monoid, Semigroup};
use newtype::{Arbitrary, Counter, Newtype};
use syntax::Entered;

use crate::watch::Kind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Kinds(u128);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Offset(usize);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Depth(usize);

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

#[derive(Clone, Copy, Debug)]
struct Pending {
    depth: Depth,
    size: Offset,
    kinds: Kinds,
}

#[derive(Debug, Default)]
struct Backward {
    done: Vec<Subtree>,
    open: Vec<Pending>,
}

impl Kinds {
    pub(crate) fn listed(kinds: impl Iterator<Item = Kind>) -> Kinds {
        Monoid::concat(kinds.map(Kinds::from))
    }

    pub(crate) fn overlap(self, other: Kinds) -> Option<()> {
        (self.0 & other.0 != 0).then_some(())
    }
}

impl From<Kind> for Kinds {
    fn from(kind: Kind) -> Kinds {
        Kinds(kind.bit())
    }
}

impl Semigroup for Kinds {
    fn combine(self, other: Kinds) -> Kinds {
        Kinds(self.0 | other.0)
    }
}

impl Monoid for Kinds {}

impl<'ast> FromIterator<Entered<'ast>> for KindIndex {
    fn from_iter<Preorder: IntoIterator<Item = Entered<'ast>>>(preorder: Preorder) -> KindIndex {
        let entered: Vec<Entered<'ast>> = preorder.into_iter().collect();
        let backward = entered.iter().rev().fold(Backward::default(), |mut backward, entry| {
            let depth = Depth::new(entry.depth());
            let deeper = backward.open.iter().rev().take_while(|pending| pending.depth > depth).count();
            let children = backward.open.split_off(backward.open.len().saturating_sub(deeper));
            let size = Offset::new(children.iter().map(|child| child.size.count()).sum());
            let kinds =
                Monoid::concat(children.iter().map(|child| child.kinds)).combine(Kinds::from(Kind::of(entry.node())));
            backward.open.push(Pending { depth, size: size.incremented(), kinds });
            backward.done.push(Subtree { descendants: size, kinds });
            backward
        });
        KindIndex(backward.done.into_iter().rev().collect())
    }
}

impl From<&syntax::File> for KindIndex {
    fn from(file: &syntax::File) -> KindIndex {
        Entered::preordered(file).into_iter().collect()
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
