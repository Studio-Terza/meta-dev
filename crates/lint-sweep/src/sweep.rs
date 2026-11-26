mod boundary;

use std::fmt;

use lint_core::{Contribute, Contributing, Joined, SourceFile, Tallied, Unjoined, Unwalked, Violation, Visits};

pub use boundary::Spelled;

pub trait Gathers<Part> {
    fn gathered(self, file: &SourceFile) -> Part;
}

pub trait Finds<'file>: fmt::Debug {
    fn found(self, file: &SourceFile) -> Found<'file>;
}

#[derive(Clone, Copy, Debug)]
pub struct Visited {
    contributor: Spelled,
    nodes: Visits,
}

impl Visited {
    #[must_use]
    pub fn contributor(&self) -> Spelled {
        self.contributor
    }

    #[must_use]
    pub fn nodes(&self) -> Visits {
        self.nodes
    }
}

#[derive(Debug)]
pub struct Found<'file> {
    violations: Vec<Violation<'file>>,
    visited: Visited,
}

impl Found<'_> {
    #[must_use]
    pub fn visited(&self) -> Visited {
        self.visited
    }
}

impl<'file> IntoIterator for Found<'file> {
    type Item = Violation<'file>;
    type IntoIter = std::vec::IntoIter<Violation<'file>>;

    fn into_iter(self) -> Self::IntoIter {
        self.violations.into_iter()
    }
}

pub trait Concludes<'file>: Contribute + fmt::Debug {
    fn violations(self, open: Self::Open, part: Self::Part) -> Vec<Violation<'file>>;
}

impl<'file, Contributor: Concludes<'file>> Finds<'file> for Contributing<Contributor> {
    fn found(self, file: &SourceFile) -> Found<'file> {
        Found::from(self.tallied(file))
    }
}

impl<'file, Contributor: Concludes<'file>> From<Tallied<Contributor>> for Found<'file> {
    fn from(tallied: Tallied<Contributor>) -> Found<'file> {
        tallied.concluded(|contributor, open, part, nodes| Found {
            violations: contributor.violations(open, part),
            visited: Visited { contributor: Spelled::naming::<Contributor>(), nodes },
        })
    }
}

pub trait Founds<'file> {
    fn founds(self, found: Vec<Option<Found<'file>>>) -> Vec<Option<Found<'file>>>;
}

impl<'file> Founds<'file> for Unjoined {
    fn founds(self, found: Vec<Option<Found<'file>>>) -> Vec<Option<Found<'file>>> {
        found
    }
}

impl<'file, Contributor: Concludes<'file>, Tail: Founds<'file>> Founds<'file>
    for Joined<Option<Tallied<Contributor>>, Tail>
{
    fn founds(self, found: Vec<Option<Found<'file>>>) -> Vec<Option<Found<'file>>> {
        self.concluded(|head, tail| tail.founds(found.into_iter().chain([head.map(Found::from)]).collect()))
    }
}

#[derive(Debug)]
pub struct Sweep;

impl Sweep {
    pub fn gathered<Part>(file: &SourceFile, gathering: impl Gathers<Part>) -> Part {
        gathering.gathered(file)
    }

    pub fn once<Part>(_: Unwalked, file: &SourceFile, gathering: impl Gathers<Part>) -> Part {
        gathering.gathered(file)
    }

    #[must_use]
    pub fn found<'file>(file: &'file SourceFile, watcher: impl Finds<'file>) -> Found<'file> {
        watcher.found(file)
    }
}
