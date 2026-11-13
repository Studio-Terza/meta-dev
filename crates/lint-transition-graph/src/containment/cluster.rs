use std::fmt;
use std::iter;

use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Level {
    Workspace,
    Crate,
    Module,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Cluster<'found> {
    krate: Option<&'found Name>,
    segments: &'found [Name],
}

impl<'found> Cluster<'found> {
    pub(crate) const WORKSPACE: Cluster<'static> = Cluster { krate: None, segments: &[] };

    pub(crate) fn module(krate: &'found Name, segments: &'found [Name]) -> Cluster<'found> {
        Cluster { krate: Some(krate), segments }
    }

    pub(crate) fn level(&self) -> Level {
        match self.krate {
            None => Level::Workspace,
            Some(_) => self.segments.first().map_or(Level::Crate, |_| Level::Module),
        }
    }

    pub(crate) fn parent(&self) -> Option<Cluster<'found>> {
        self.krate.map(|krate| {
            self.segments.split_last().map_or(Cluster::WORKSPACE, |(_, outer)| Cluster::module(krate, outer))
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
        let own = self.segments.last().or(self.krate);
        own.filter(|candidate| *candidate == name).map(|_| self)
    }

    pub(crate) fn krate(&self) -> Option<&'found Name> {
        self.krate
    }

    pub(crate) fn segments(&self) -> &'found [Name] {
        self.segments
    }
}

#[derive(Debug)]
enum Whereabouts<'found> {
    Workspace,
    Crate(&'found Name),
    Module(&'found Name, Segments<'found>),
}

impl<'found> Whereabouts<'found> {
    fn locating(cluster: &Cluster<'found>) -> Whereabouts<'found> {
        match cluster.krate {
            None => Whereabouts::Workspace,
            Some(krate) => Whereabouts::within(krate, cluster.segments),
        }
    }

    fn within(krate: &'found Name, segments: &'found [Name]) -> Whereabouts<'found> {
        match segments.first() {
            None => Whereabouts::Crate(krate),
            Some(_) => Whereabouts::Module(krate, Segments(segments)),
        }
    }
}

impl fmt::Display for Cluster<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match Whereabouts::locating(self) {
            Whereabouts::Workspace => write!(formatter, "the workspace"),
            Whereabouts::Crate(krate) => write!(formatter, "crate {krate}"),
            Whereabouts::Module(krate, segments) => write!(formatter, "{krate}::{segments}"),
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

impl fmt::Display for Segments<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let segments: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(formatter, "{}", segments.join("::"))
    }
}
