use std::fmt;
use std::iter;

use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Workspace,
    Crate,
    Module,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nest<'found, Crate> {
    krate: Option<Crate>,
    segments: &'found [Name],
}

#[derive(Clone, Copy, Debug)]
struct Segments<'found>(&'found [Name]);

impl<'found, Crate: Copy + PartialEq> Nest<'found, Crate> {
    pub const WORKSPACE: Nest<'static, Crate> = Nest { krate: None, segments: &[] };

    #[must_use]
    pub fn module(krate: Crate, segments: &'found [Name]) -> Nest<'found, Crate> {
        Nest { krate: Some(krate), segments }
    }

    #[must_use]
    pub fn level(&self) -> Level {
        self.krate.map_or(Level::Workspace, |_| self.segments.first().map_or(Level::Crate, |_| Level::Module))
    }

    #[must_use]
    pub fn parent(&self) -> Option<Nest<'found, Crate>> {
        self.krate.map(|krate| {
            self.segments
                .split_last()
                .map_or(Nest { krate: None, segments: &[] }, |(_, outer)| Nest::module(krate, outer))
        })
    }

    pub fn lineage(self) -> impl Iterator<Item = Nest<'found, Crate>> {
        iter::successors(Some(self), Nest::parent)
    }

    #[must_use]
    pub fn holding(self, inner: Nest<'found, Crate>) -> Option<Nest<'found, Crate>> {
        inner.lineage().find(|outer| *outer == self)
    }

    #[must_use]
    pub fn common(self, other: Nest<'found, Crate>) -> Nest<'found, Crate> {
        self.lineage().find(|outer| outer.holding(other).is_some()).unwrap_or(Nest { krate: None, segments: &[] })
    }

    #[must_use]
    pub fn krate(&self) -> Option<Crate> {
        self.krate
    }
}

impl<'found, Crate> From<Nest<'found, Crate>> for &'found [Name] {
    fn from(nest: Nest<'found, Crate>) -> &'found [Name] {
        nest.segments
    }
}

impl<Crate: Copy + PartialEq + fmt::Display> fmt::Display for Nest<'_, Crate> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.level() {
            Level::Workspace => write!(formatter, "the workspace"),
            Level::Crate => self.krate.map_or(Ok(()), |krate| write!(formatter, "crate {krate}")),
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

impl<'found> Nest<'found, &'found Name> {
    #[must_use]
    pub fn named(self, name: &Name) -> Option<Nest<'found, &'found Name>> {
        let own = self.segments.last().or(self.krate);
        own.filter(|candidate| *candidate == name).map(|_| self)
    }
}
