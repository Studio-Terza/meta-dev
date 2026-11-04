use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::iter;
use std::ops::Not;

use lint_layout::Name;
use newtype::Newtype;

use super::super::member::Member;
use super::super::{Containment, Toward};
use super::listing::{Listed, Placed, Spelled};

#[derive(Debug)]
pub(super) struct Route<'view, 'found, 'file> {
    view: &'view Containment<'found, 'file>,
    from: &'view Name,
    to: &'view Name,
    listed: Listed,
}

#[derive(Debug)]
struct Frontier<'found> {
    reached: BTreeMap<Member<'found>, Member<'found>>,
    layer: BTreeSet<Member<'found>>,
}

impl<'view, 'found, 'file> Route<'view, 'found, 'file> {
    pub(super) fn new(
        view: &'view Containment<'found, 'file>,
        from: &'view Name,
        to: &'view Name,
        listed: Listed,
    ) -> Route<'view, 'found, 'file> {
        Route { view, from, to, listed }
    }

    fn found(&self, start: Member<'found>) -> Option<Vec<Member<'found>>> {
        let frontiers = iter::successors(Some(Frontier::from(start)), |frontier| frontier.widened(self.view));
        frontiers.filter(|frontier| frontier.layer.first().is_some_and(|first| *first != start)).find_map(|frontier| {
            frontier.layer.iter().find(|member| member.name() == self.to).map(|end| frontier.walked(*end))
        })
    }
}

impl<'found> From<Member<'found>> for Frontier<'found> {
    fn from(start: Member<'found>) -> Frontier<'found> {
        Frontier { reached: BTreeMap::from([(start, start)]), layer: BTreeSet::from([start]) }
    }
}

impl<'found> Frontier<'found> {
    fn widened(&self, view: &Containment<'found, '_>) -> Option<Frontier<'found>> {
        let next: BTreeMap<Member<'found>, Member<'found>> = self
            .layer
            .iter()
            .flat_map(|user| view.adjacent(*user, Toward::Used).map(|used| (used, *user)))
            .filter(|(used, _)| self.reached.contains_key(used).not())
            .collect();
        let layer: BTreeSet<Member<'found>> = next.keys().copied().collect();
        let reached = self.reached.iter().map(|(used, user)| (*used, *user)).chain(next).collect();
        Some(Frontier { reached, layer }).filter(|frontier| frontier.layer.first().is_some())
    }

    fn walked(&self, end: Member<'found>) -> Vec<Member<'found>> {
        let back =
            iter::successors(Some(end), |member| self.reached.get(member).copied().filter(|user| user != member));
        let mut route: Vec<Member<'found>> = back.collect();
        route.reverse();
        route
    }
}

impl fmt::Display for Route<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let route = self.view.named(self.from).find_map(|start| self.found(start)).unwrap_or_default();
        route.first().map_or_else(|| writeln!(formatter, "  none"), |_| Ok(()))?;
        route
            .iter()
            .take(self.listed.into_inner())
            .try_for_each(|member| writeln!(formatter, "  {}", Placed::new(self.view, *member, Spelled::Path)))?;
        self.listed.more(&route).map_or(Ok(()), |more| writeln!(formatter, "  and {more} more"))
    }
}
