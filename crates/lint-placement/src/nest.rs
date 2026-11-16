use std::fmt;
use std::iter;

use lint_layout::Name;

use crate::crate_root::CrateRoot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Workspace,
    Crate,
    Module,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Krate<'found> {
    Named(&'found Name),
    Rooted(&'found CrateRoot),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nest<'found> {
    krate: Option<Krate<'found>>,
    segments: &'found [Name],
}

#[derive(Clone, Copy, Debug)]
struct Segments<'found>(&'found [Name]);

impl<'found> Krate<'found> {
    #[must_use]
    pub fn name(self) -> Option<&'found Name> {
        match self {
            Krate::Named(name) => Some(name),
            Krate::Rooted(_) => None,
        }
    }
}

impl<'found> Nest<'found> {
    pub const WORKSPACE: Nest<'static> = Nest { krate: None, segments: &[] };

    #[must_use]
    pub fn module(krate: Krate<'found>, segments: &'found [Name]) -> Nest<'found> {
        Nest { krate: Some(krate), segments }
    }

    #[must_use]
    pub fn level(&self) -> Level {
        self.krate.map_or(Level::Workspace, |_| self.segments.first().map_or(Level::Crate, |_| Level::Module))
    }

    #[must_use]
    pub fn parent(&self) -> Option<Nest<'found>> {
        self.krate.map(|krate| {
            self.segments
                .split_last()
                .map_or(Nest { krate: None, segments: &[] }, |(_, outer)| Nest::module(krate, outer))
        })
    }

    pub fn lineage(self) -> impl Iterator<Item = Nest<'found>> {
        iter::successors(Some(self), Nest::parent)
    }

    #[must_use]
    pub fn holding(self, inner: Nest<'found>) -> Option<Nest<'found>> {
        inner.lineage().find(|outer| *outer == self)
    }

    #[must_use]
    pub fn common(self, other: Nest<'found>) -> Nest<'found> {
        self.lineage().find(|outer| outer.holding(other).is_some()).unwrap_or(Nest { krate: None, segments: &[] })
    }

    #[must_use]
    pub fn named(self, name: &Name) -> Option<Nest<'found>> {
        let own = self.segments.last().or_else(|| self.krate.and_then(Krate::name));
        own.filter(|candidate| *candidate == name).map(|_| self)
    }

    #[must_use]
    pub fn krate(&self) -> Option<Krate<'found>> {
        self.krate
    }
}

impl<'found> From<Nest<'found>> for &'found [Name] {
    fn from(nest: Nest<'found>) -> &'found [Name] {
        nest.segments
    }
}

impl fmt::Display for Krate<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Krate::Named(name) => write!(formatter, "{name}"),
            Krate::Rooted(_) => write!(formatter, "crate"),
        }
    }
}

impl Nest<'_> {
    fn crate_shown(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.krate.and_then(Krate::name).map_or(Ok(()), |name| write!(formatter, " {name}"))
    }
}

impl fmt::Display for Nest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.level() {
            Level::Workspace => write!(formatter, "the workspace"),
            Level::Crate => self.crate_shown(formatter),
            Level::Module => {
                self.krate.map_or(Ok(()), |krate| write!(formatter, "{krate}::{}", Segments(self.segments)))
            }
        }
    }
}

impl fmt::Display for Segments<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let segments: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(formatter, "{}", segments.join("::"))
    }
}
