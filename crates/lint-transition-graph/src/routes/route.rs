use std::collections::BTreeSet;
use std::fmt;
use std::ptr;

use equivalence::{Group, Holes};
use lint_core::{Fix, Site, Violation};
use lint_layout::Name;
use newtype::Counter;

use super::leg::Leg;
use super::stops::Stops;
use super::walk::Walk;
use crate::count::Count;
use crate::edge::Edge;
use crate::listed::Listed;
use crate::share::Share;

#[derive(Debug)]
pub(super) struct Route<'walk, 'graph, 'found, 'file> {
    group: Group<Leg<'walk, 'graph, 'found, 'file>>,
    places: Vec<Site<'file>>,
    joined: Option<&'graph Edge<'found, 'file>>,
}

impl<'walk, 'graph, 'found, 'file> Route<'walk, 'graph, 'found, 'file> {
    const LEAST_PLACES: usize = 2;

    pub(super) fn of_group(
        group: Group<Leg<'walk, 'graph, 'found, 'file>>,
        stops: &Stops<'graph, 'found, 'file>,
    ) -> Option<Route<'walk, 'graph, 'found, 'file>> {
        let places: Vec<Site<'file>> =
            group.sites().into_iter().copied().collect::<BTreeSet<Site<'file>>>().into_iter().collect();
        let called = group.members().first().map(Leg::called).unwrap_or_default();
        let joined = called
            .first()
            .zip(called.last())
            .and_then(|(first, last)| stops.joining(first, last))
            .filter(|joined| Self::outside(&group, joined).is_some());
        Some(Route { group, places, joined }).filter(|route| route.places.len() >= Self::LEAST_PLACES)
    }

    fn outside<'edge>(
        group: &Group<Leg<'walk, 'graph, 'found, 'file>>,
        joined: &'edge Edge<'found, 'file>,
    ) -> Option<&'edge Edge<'found, 'file>> {
        let walkers = group.members().iter().map(|member| member.walk().edge());
        let called = group.members().iter().flat_map(|member| member.called().iter().copied());
        walkers.chain(called).find(|edge| ptr::eq(*edge, joined)).map_or(Some(joined), |_| None)
    }

    fn holes(&self) -> Holes {
        let named: Vec<Vec<&Name>> = self
            .group
            .members()
            .iter()
            .map(|member| {
                member.called().iter().map(|edge| AsRef::<Name>::as_ref(edge.transition().function())).collect()
            })
            .collect();
        let spelled: Vec<&[&Name]> = named.iter().map(Vec::as_slice).collect();
        Holes::between(&spelled)
    }
}

impl<'file> From<&Route<'_, '_, '_, 'file>> for Option<Violation<'file>> {
    fn from(route: &Route<'_, '_, '_, 'file>) -> Option<Violation<'file>> {
        route.places.first().map(|site| Violation::new(*site, route))
    }
}

impl fmt::Display for Route<'_, '_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let called = self.group.members().first().map(Leg::called).unwrap_or_default();
        let route = Listed::<_, ','>::joined(called.iter().map(|edge| edge.transition()));
        let others = Listed::<_, ','>::joined(self.places.iter().skip(1));
        write!(
            formatter,
            "a route of {} hops is walked in {} places: {route}; also at {others}; callee names differing: {}",
            Count::new(called.len()),
            Count::new(self.places.len()),
            self.holes()
        )?;
        self.joined.map_or(Ok(()), |edge| write!(formatter, "; {} already joins its ends", edge.transition()))
    }
}

impl lint_core::Finding for &Route<'_, '_, '_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "name the route once, as one function from its first inputs to its last output, and call it from each place; where the places mean different things, give them different types",
        )
    }
}

#[derive(Debug)]
pub(super) struct Tally {
    walks: Count,
    calls: Count,
    resolved: Count,
    routes: Count,
}

impl Tally {
    pub(super) fn counted(walks: &[Walk<'_, '_, '_>], routes: &[Route<'_, '_, '_, '_>]) -> Tally {
        Tally {
            walks: Count::new(walks.len()),
            calls: Count::total(walks.iter().map(|walk| Count::new(walk.edge().transition().function().calls().len()))),
            resolved: Count::total(walks.iter().map(Walk::resolved)),
            routes: Count::new(routes.len()),
        }
    }

    pub(super) fn placed(self, first: Option<Site<'_>>) -> Option<Violation<'_>> {
        first.map(|site| Violation::new(site, self))
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {} written bodies make {} calls, {} resolved to a workspace edge, share unresolved {}; {} repeated routes",
            self.walks,
            self.calls,
            self.resolved,
            Share::between(Count::new(self.calls.count().saturating_sub(self.resolved.count())), self.calls),
            self.routes
        )
    }
}

impl lint_core::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("name each repeated route once and call it from each place")
    }
}
