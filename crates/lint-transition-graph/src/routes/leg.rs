use std::ptr;

use equivalence::{Keyed, Nested};
use lint_core::Site;

use super::stops::Stop;
use super::walk::Walk;
use crate::count::Count;
use crate::edge::Edge;

#[derive(Clone, Copy, Debug)]
pub(super) struct Leg<'walk, 'graph, 'found, 'file> {
    walk: &'walk Walk<'graph, 'found, 'file>,
    called: &'walk [&'graph Edge<'found, 'file>],
    stops: &'walk [Stop],
    from: Count,
    to: Count,
}

impl<'walk, 'graph, 'found, 'file> Leg<'walk, 'graph, 'found, 'file> {
    pub(super) fn new(
        walk: &'walk Walk<'graph, 'found, 'file>,
        from: Count,
        to: Count,
    ) -> Leg<'walk, 'graph, 'found, 'file> {
        Leg { walk, called: walk.called(from, to), stops: walk.stops(from, to), from, to }
    }

    pub(super) fn called(&self) -> &'walk [&'graph Edge<'found, 'file>] {
        self.called
    }

    pub(super) fn walk(&self) -> &'walk Walk<'graph, 'found, 'file> {
        self.walk
    }
}

impl<'walk> Keyed for Leg<'walk, '_, '_, '_> {
    type Key = &'walk [Stop];

    fn key(&self) -> &'walk [Stop] {
        self.stops
    }
}

impl Nested for Leg<'_, '_, '_, '_> {
    fn lies_within<'outer>(&self, outer: &'outer Self) -> Option<&'outer Self> {
        Some(outer)
            .filter(|outer| ptr::eq(self.walk, outer.walk))
            .filter(|outer| outer.from <= self.from)
            .filter(|outer| self.to <= outer.to)
    }
}

impl<'file> AsRef<Site<'file>> for Leg<'_, '_, '_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        self.walk.edge().transition().as_ref()
    }
}
