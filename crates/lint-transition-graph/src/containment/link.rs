use std::collections::BTreeSet;
use std::fmt;

use lint_placement::CrateName;

use super::member::Member;

const ENDS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Link<'found> {
    from: Member<'found>,
    to: Member<'found>,
}

impl<'found> Link<'found> {
    pub(crate) fn new(from: Member<'found>, to: Member<'found>) -> Link<'found> {
        Link { from, to }
    }

    pub(crate) fn held(own: &CrateName, holder: Member<'found>, end: Member<'found>) -> Link<'found> {
        let elsewhere = holder.cluster().krate().filter(|krate| *krate != own);
        elsewhere.map_or(Link { from: holder, to: end }, |_| Link { from: end, to: holder })
    }

    pub(crate) fn ends(&self) -> [Member<'found>; ENDS] {
        [self.from, self.to]
    }

    pub(crate) fn reversed(self) -> Link<'found> {
        Link { from: self.to, to: self.from }
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
