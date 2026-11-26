mod boundary;

use std::fmt;

use lint_core::{Contribute, Contributing, SourceFile, Unwalked, Violation, Visits};

pub use boundary::Spelled;

pub trait Gathers<Part> {
    fn gathered(self, file: &SourceFile) -> Part;
}

pub trait Finds<'file>: fmt::Debug {
    fn found(self: Box<Self>, file: &SourceFile) -> Found<'file>;
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
    fn found(self: Box<Self>, file: &SourceFile) -> Found<'file> {
        self.tallied(file).concluded(|contributor, open, part, nodes| Found {
            violations: contributor.violations(open, part),
            visited: Visited { contributor: Spelled::naming::<Contributor>(), nodes },
        })
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
    pub fn found<'file>(file: &'file SourceFile, watcher: Box<dyn Finds<'file> + '_>) -> Found<'file> {
        watcher.found(file)
    }
}

#[derive(Debug, Default)]
pub struct Watchers<'file>(Vec<Box<dyn Finds<'file> + 'file>>);

impl<'file> FromIterator<Box<dyn Finds<'file> + 'file>> for Watchers<'file> {
    fn from_iter<Found: IntoIterator<Item = Box<dyn Finds<'file> + 'file>>>(found: Found) -> Watchers<'file> {
        Watchers(found.into_iter().collect())
    }
}

impl<'file> Gathers<Vec<Violation<'file>>> for Watchers<'file> {
    fn gathered(self, file: &SourceFile) -> Vec<Violation<'file>> {
        self.0.into_iter().flat_map(|watcher| watcher.found(file)).collect()
    }
}
