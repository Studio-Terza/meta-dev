use algebra::{Monoid, Semigroup};

use crate::watch::Descend;
use crate::watch::{Contribute, Kind};

#[derive(Debug)]
pub struct Contributed<Open, Part> {
    within: Contribution<Open, Part>,
    descend: Descend,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Contribution<Open, Part> {
    open: Open,
    part: Part,
}

#[derive(Debug)]
pub struct Contributing<Contributor: Contribute> {
    contributor: Contributor,
    kinds: Vec<Kind>,
    frames: Vec<Contribution<Contributor::Open, Contributor::Part>>,
}

impl<Open: Monoid, Part: Monoid> Contributed<Open, Part> {
    #[must_use]
    pub fn new(open: Open, part: Part, descend: Descend) -> Contributed<Open, Part> {
        Contributed { within: Contribution { open, part }, descend }
    }

    #[must_use]
    pub fn parted(part: Part) -> Contributed<Open, Part> {
        Contributed::new(Open::identity(), part, Descend::Into)
    }

    #[must_use]
    pub fn descending(descend: Descend) -> Contributed<Open, Part> {
        Contributed::new(Open::identity(), Part::identity(), descend)
    }
}

impl<Open, Part> Contribution<Open, Part> {
    pub fn concluded<Out>(self, into: impl FnOnce(Open, Part) -> Out) -> Out {
        into(self.open, self.part)
    }
}

impl<Open: Semigroup, Part: Semigroup> Semigroup for Contribution<Open, Part> {
    fn combine(self, other: Contribution<Open, Part>) -> Contribution<Open, Part> {
        Contribution { open: self.open.combine(other.open), part: self.part.combine(other.part) }
    }
}

impl<Open: Monoid, Part: Monoid> Monoid for Contribution<Open, Part> {
    fn identity() -> Contribution<Open, Part> {
        Contribution { open: Open::identity(), part: Part::identity() }
    }
}

impl<Contributor: Contribute> Contributing<Contributor> {
    pub fn new(contributor: Contributor) -> Contributing<Contributor> {
        let kinds = contributor.kinds().iter().chain(contributor.scopes()).copied().collect();
        Contributing { contributor, kinds, frames: vec![Contribution::identity()] }
    }

    pub fn concluded<Out>(self, into: impl FnOnce(Contributor::Open, Contributor::Part) -> Out) -> Out {
        let Contribution { open, part } = Monoid::concat(self.frames);
        into(open, part)
    }

    pub(crate) fn watched(&self) -> &[Kind] {
        &self.kinds
    }

    pub(crate) fn entered(
        &mut self,
        kind: Kind,
        contributed: impl FnOnce(&Contributor) -> Contributed<Contributor::Open, Contributor::Part>,
    ) -> Descend {
        let Contributed { within, descend } = contributed(&self.contributor);
        let opened = self.contributor.scopes().contains(&kind).then(Contribution::identity);
        let top = opened.unwrap_or_else(|| self.frames.pop().unwrap_or_else(Contribution::identity));
        self.frames.push(top.combine(within));
        descend
    }

    pub(crate) fn left(
        &mut self,
        kind: Kind,
        closed: impl FnOnce(&Contributor, Contribution<Contributor::Open, Contributor::Part>) -> Contributor::Part,
    ) {
        let scope = self.contributor.scopes().iter().find(|scope| **scope == kind);
        let inner = scope.and_then(|_| self.frames.pop());
        let closing = inner.map(|within| closed(&self.contributor, within));
        let merged = closing.map(|part| {
            let top = self.frames.pop().unwrap_or_else(Contribution::identity);
            top.combine(Contribution { open: Contributor::Open::identity(), part })
        });
        self.frames.extend(merged);
    }
}
