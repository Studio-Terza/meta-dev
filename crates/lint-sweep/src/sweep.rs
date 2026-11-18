mod boundary;

use std::fmt;

use lint_core::{Contribute, Contributing, SourceFile, Unwalked, Violation};

pub use boundary::Member;

pub trait Gathers<Part> {
    fn gathered(self, file: &SourceFile) -> Part;
}

pub trait Finds<'file>: fmt::Debug {
    fn found(self: Box<Self>, file: &SourceFile) -> Vec<Violation<'file>>;
}

pub trait Sorts<'file>: fmt::Debug {
    fn sorted(self: Box<Self>, file: &SourceFile) -> Ranked<'file>;
}

pub trait Concludes<'file>: Contribute + fmt::Debug {
    fn violations(self, open: Self::Open, part: Self::Part) -> Vec<Violation<'file>>;
}

impl<'file, Contributor: Concludes<'file>> Finds<'file> for Contributing<Contributor> {
    fn found(self: Box<Self>, file: &SourceFile) -> Vec<Violation<'file>> {
        self.concluded(file, Contributor::violations)
    }
}

#[derive(Debug)]
pub struct Ranked<'file>(Vec<Vec<Violation<'file>>>);

impl<'file> From<Vec<Vec<Violation<'file>>>> for Ranked<'file> {
    fn from(lists: Vec<Vec<Violation<'file>>>) -> Ranked<'file> {
        Ranked(lists)
    }
}

impl<'file> From<Ranked<'file>> for Vec<Vec<Violation<'file>>> {
    fn from(ranked: Ranked<'file>) -> Vec<Vec<Violation<'file>>> {
        ranked.0
    }
}

pub trait Ranks<'file>: Contribute + fmt::Debug {
    fn ranked(self, open: Self::Open, part: Self::Part) -> Ranked<'file>;
}

impl<'file, Contributor: Ranks<'file>> Sorts<'file> for Contributing<Contributor> {
    fn sorted(self: Box<Self>, file: &SourceFile) -> Ranked<'file> {
        self.concluded(file, Contributor::ranked)
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
    pub fn found<'file>(file: &'file SourceFile, watcher: Box<dyn Finds<'file> + '_>) -> Vec<Violation<'file>> {
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
