use std::collections::BTreeSet;
use std::fmt;

use super::member::Member;

const ENDS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Link<'found> {
    from: Member<'found>,
    to: Member<'found>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Reversed<'found>(Link<'found>);

impl<'found> From<Link<'found>> for Reversed<'found> {
    fn from(link: Link<'found>) -> Reversed<'found> {
        Reversed(link)
    }
}

impl<'found> From<Reversed<'found>> for Link<'found> {
    fn from(reversed: Reversed<'found>) -> Link<'found> {
        Link { from: reversed.0.to, to: reversed.0.from }
    }
}

impl<'found> Link<'found> {
    pub(crate) fn new(from: Member<'found>, to: Member<'found>) -> Link<'found> {
        Link { from, to }
    }

    pub(crate) fn ends(&self) -> [Member<'found>; ENDS] {
        [self.from, self.to]
    }

    pub(crate) fn star(linked: &BTreeSet<Member<'found>>) -> Vec<Link<'found>> {
        linked
            .first()
            .copied()
            .map_or_else(Vec::new, |first| linked.iter().map(|other| Link::new(first, *other)).collect())
    }

    pub(crate) fn apart(self) -> Option<Link<'found>> {
        Some(self).filter(|link| link.from != link.to)
    }
}

impl fmt::Display for Link<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}", self.from, self.to)
    }
}
