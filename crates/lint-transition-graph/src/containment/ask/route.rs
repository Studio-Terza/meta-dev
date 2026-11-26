use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::iter;
use std::ops::Not;

use lint_layout::Name;
use newtype::Wrapping;

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

    fn found(&self) -> Vec<Member<'found>> {
        self.view.named(self.from).collect::<Frontier<'found>>().reaching(self.view, self.to)
    }
}

impl<'found> FromIterator<Member<'found>> for Frontier<'found> {
    fn from_iter<Starts: IntoIterator<Item = Member<'found>>>(starts: Starts) -> Frontier<'found> {
        let layer: BTreeSet<Member<'found>> = starts.into_iter().collect();
        Frontier { reached: layer.iter().map(|start| (*start, *start)).collect(), layer }
    }
}

#[derive(Debug)]
struct Reaching<'view, 'found, 'file> {
    view: &'view Containment<'found, 'file>,
    to: &'view Name,
    frontier: Option<Frontier<'found>>,
}

impl<'found> Iterator for Reaching<'_, 'found, '_> {
    type Item = Option<Vec<Member<'found>>>;

    fn next(&mut self) -> Option<Option<Vec<Member<'found>>>> {
        let frontier = self.frontier.take()?;
        let view = self.view;
        let next: BTreeMap<Member<'found>, Member<'found>> = frontier
            .layer
            .iter()
            .flat_map(|user| view.adjacent(*user, Toward::Used).map(|used| (used, *user)))
            .filter(|(used, _)| frontier.reached.contains_key(used).not())
            .collect();
        let layer: BTreeSet<Member<'found>> = next.keys().copied().collect();
        let reached = frontier.reached.into_iter().chain(next).collect();
        let widened = Some(Frontier { reached, layer }).filter(|grown| grown.layer.first().is_some())?;
        let ended = widened.ended(self.to);
        self.frontier = Some(widened);
        Some(ended)
    }
}

impl<'found> Frontier<'found> {
    fn reaching(self, view: &Containment<'found, '_>, to: &Name) -> Vec<Member<'found>> {
        Reaching { view, to, frontier: Some(self) }.find_map(|ended| ended).unwrap_or_default()
    }

    fn ended(&self, to: &Name) -> Option<Vec<Member<'found>>> {
        self.layer.iter().find(|member| <&Name>::from(**member) == to).map(|end| self.walked(*end))
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
        let route = self.found();
        route.first().map_or_else(|| writeln!(formatter, "  none"), |_| Ok(()))?;
        route
            .iter()
            .take(self.listed.into_inner())
            .try_for_each(|member| writeln!(formatter, "  {}", Placed::new(self.view, *member, Spelled::Path)))?;
        self.listed.more(&route).map_or(Ok(()), |more| writeln!(formatter, "  and {more} more"))
    }
}
