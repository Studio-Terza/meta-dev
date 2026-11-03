use std::fmt;
use std::iter;

use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Level {
    Workspace,
    Crate,
    Module,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Cluster<'found> {
    krate: Option<&'found CrateName>,
    segments: &'found [Name],
}

impl<'found> Cluster<'found> {
    pub(crate) const WORKSPACE: Cluster<'static> = Cluster { krate: None, segments: &[] };

    pub(crate) fn module(krate: &'found CrateName, segments: &'found [Name]) -> Cluster<'found> {
        Cluster { krate: Some(krate), segments }
    }

    pub(crate) fn level(&self) -> Level {
        match (self.krate, self.segments.first()) {
            (None, _) => Level::Workspace,
            (Some(_), None) => Level::Crate,
            (Some(_), Some(_)) => Level::Module,
        }
    }

    pub(crate) fn parent(&self) -> Option<Cluster<'found>> {
        match (self.krate, self.segments.split_last()) {
            (Some(krate), Some((_, outer))) => Some(Cluster::module(krate, outer)),
            (Some(_), None) => Some(Cluster::WORKSPACE),
            (None, _) => None,
        }
    }

    pub(crate) fn lineage(self) -> impl Iterator<Item = Cluster<'found>> {
        iter::successors(Some(self), Cluster::parent)
    }

    pub(crate) fn holding(self, inner: Cluster<'found>) -> Option<Cluster<'found>> {
        inner.lineage().find(|outer| *outer == self)
    }

    pub(crate) fn common(self, other: Cluster<'found>) -> Cluster<'found> {
        self.lineage().find(|outer| outer.holding(other).is_some()).unwrap_or(Cluster::WORKSPACE)
    }

    pub(crate) fn part_holding(self, inner: Cluster<'found>) -> Cluster<'found> {
        inner.lineage().find(|outer| outer.parent() == Some(self)).unwrap_or(self)
    }
}

impl fmt::Display for Cluster<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.krate, self.segments.first()) {
            (None, _) => write!(formatter, "the workspace"),
            (Some(krate), None) => write!(formatter, "crate {krate}"),
            (Some(krate), Some(_)) => write!(formatter, "{krate}::{}", Segments(self.segments)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Part<'found> {
    whole: Cluster<'found>,
    part: Cluster<'found>,
}

impl<'found> Part<'found> {
    pub(crate) fn of(whole: Cluster<'found>, inner: Cluster<'found>) -> Part<'found> {
        Part { whole, part: whole.part_holding(inner) }
    }

    pub(crate) fn nested(&self) -> Option<Cluster<'found>> {
        Some(self.part).filter(|part| *part != self.whole)
    }
}

impl fmt::Display for Part<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.nested() {
            Some(part) => write!(formatter, "{part}"),
            None => write!(formatter, "{} own file", self.whole),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Segments<'found>(pub(crate) &'found [Name]);

impl fmt::Display for Segments<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let segments: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(formatter, "{}", segments.join("::"))
    }
}
