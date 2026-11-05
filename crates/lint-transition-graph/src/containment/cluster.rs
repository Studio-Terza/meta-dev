use std::borrow::Borrow;
use std::fmt;
use std::iter;

use super::unit::Unit;
use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Level {
    Workspace,
    Crate,
    Module,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Cluster<'found> {
    unit: Option<Unit<'found>>,
    segments: &'found [Name],
}

impl<'found> Cluster<'found> {
    pub(crate) const WORKSPACE: Cluster<'static> = Cluster { unit: None, segments: &[] };

    pub(crate) fn module(unit: Unit<'found>, segments: &'found [Name]) -> Cluster<'found> {
        Cluster { unit: Some(unit), segments }
    }

    pub(crate) fn level(&self) -> Level {
        match self.unit {
            None => Level::Workspace,
            Some(_) => self.segments.first().map_or(Level::Crate, |_| Level::Module),
        }
    }

    pub(crate) fn parent(&self) -> Option<Cluster<'found>> {
        self.unit.map(|unit| {
            self.segments.split_last().map_or(Cluster::WORKSPACE, |(_, outer)| Cluster::module(unit, outer))
        })
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

    pub(crate) fn named(self, name: &Name) -> Option<Cluster<'found>> {
        let own = self.segments.last().or_else(|| self.unit.map(|unit| unit.krate().borrow()));
        own.filter(|own| *own == name).map(|_| self)
    }

    pub(crate) fn unit(&self) -> Option<Unit<'found>> {
        self.unit
    }

    pub(crate) fn segments(&self) -> &'found [Name] {
        self.segments
    }
}

impl fmt::Display for Cluster<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.unit {
            None => write!(formatter, "the workspace"),
            Some(unit) => Segments(self.segments).named_in(unit, formatter),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Part<'found> {
    whole: Cluster<'found>,
    part: Cluster<'found>,
}

impl<'found> Part<'found> {
    pub(crate) fn toward(whole: Cluster<'found>, inner: Cluster<'found>) -> Part<'found> {
        Part { whole, part: inner.lineage().find(|outer| outer.parent() == Some(whole)).unwrap_or(whole) }
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
pub(crate) struct Segments<'found>(&'found [Name]);

impl Segments<'_> {
    fn named_in(self, unit: Unit<'_>, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.first() {
            None => write!(formatter, "{unit}"),
            Some(_) => write!(formatter, "{}::{self}", unit.krate()),
        }
    }
}

impl fmt::Display for Segments<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let segments: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(formatter, "{}", segments.join("::"))
    }
}
