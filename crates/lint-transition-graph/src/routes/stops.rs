use std::collections::BTreeMap;
use std::fmt;
use std::ptr;

use equivalence::Group;
use lint_layout::Name;
use newtype::{Arbitrary, Newtype};

use super::leg::Leg;
use super::walk::Walk;
use crate::edge::Edge;
use crate::end::{End, Hop};
use crate::forms::Written;
use crate::transitions::Wrapping;
use graph::CallOwner;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Stop(usize);

#[derive(Clone, Copy, Debug)]
pub(super) struct Joined<'graph, 'found, 'file>(&'graph Edge<'found, 'file>);

#[derive(Clone, Copy, Debug)]
pub(super) struct Outside<'graph, 'found, 'file>(&'graph Edge<'found, 'file>);

impl<'graph, 'found, 'file> Joined<'graph, 'found, 'file> {
    pub(super) fn outside(
        self,
        group: &Group<Leg<'_, 'graph, 'found, 'file>>,
    ) -> Option<Outside<'graph, 'found, 'file>> {
        let walkers = group.members().iter().map(|member| member.walk().edge());
        let called = group.members().iter().flat_map(|member| member.called().iter().copied());
        walkers.chain(called).find(|edge| ptr::eq(*edge, self.0)).map_or(Some(Outside(self.0)), |_| None)
    }
}

impl fmt::Display for Outside<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "; {} already joins its ends", self.0.transition())
    }
}

#[derive(Debug)]
pub(super) struct Stops<'graph, 'found, 'file> {
    named: BTreeMap<&'graph Name, Vec<&'graph Edge<'found, 'file>>>,
    hops: BTreeMap<Hop<'found>, Stop>,
    joins: Vec<&'graph Edge<'found, 'file>>,
}

impl<'graph, 'found, 'file> Stops<'graph, 'found, 'file> {
    pub(super) fn indexed(edges: &'graph [Edge<'found, 'file>]) -> Stops<'graph, 'found, 'file> {
        let joins: Vec<&'graph Edge<'found, 'file>> =
            edges.iter().collect::<Written<'graph, 'found, 'file>>().into_iter().collect();
        let named = edges.iter().fold(BTreeMap::new(), |mut named: BTreeMap<_, Vec<_>>, edge| {
            named.entry(AsRef::<Name>::as_ref(edge.transition().function())).or_default().push(edge);
            named
        });
        let hops = joins.iter().filter_map(|edge| Option::<Hop<'found>>::from(edge.ends())).fold(
            BTreeMap::new(),
            |mut hops, hop| {
                let next = Stop::new(hops.len());
                hops.entry(hop).or_insert(next);
                hops
            },
        );
        Stops { named, hops, joins }
    }

    pub(super) fn walked(&self, edge: &'graph Edge<'found, 'file>) -> Walk<'graph, 'found, 'file> {
        let walker = edge.transition().context().selfish().subject().map(AsRef::<Name>::as_ref);
        let resolved = edge.transition().function().calls().iter().fold(Vec::new(), |mut resolved, callee| {
            let earlier: Vec<Option<&Name>> = resolved
                .iter()
                .copied()
                .map(|found: Option<&'graph Edge<'found, 'file>>| found.and_then(Self::output))
                .collect();
            let owner = callee.owner(walker, &earlier);
            resolved.push(self.resolved(callee.as_ref(), owner));
            resolved
        });
        let called: Vec<&'graph Edge<'found, 'file>> =
            resolved.into_iter().flatten().filter(|called| self.stop(called).is_some()).collect();
        let stops = called.iter().filter_map(|callee| self.stop(callee)).collect();
        Walk::new(edge, called, stops)
    }

    pub(super) fn joining(
        &self,
        first: &Edge<'found, 'file>,
        last: &Edge<'found, 'file>,
    ) -> Option<Joined<'graph, 'found, 'file>> {
        let ends = Option::<Hop<'found>>::from(first.ends()).zip(Option::<Hop<'found>>::from(last.ends()));
        ends.and_then(|(from, to)| self.joins.iter().copied().find_map(|edge| Self::closes(edge, &from, &to)))
    }

    fn output(edge: &'graph Edge<'found, 'file>) -> Option<&'graph Name> {
        (edge.transition().function().wrapping() == Wrapping::Plain)
            .then(|| AsRef::<End<'found>>::as_ref(edge.ends()).as_ref())
    }

    fn closes(
        edge: &'graph Edge<'found, 'file>,
        from: &Hop<'found>,
        to: &Hop<'found>,
    ) -> Option<Joined<'graph, 'found, 'file>> {
        let hop = Option::<Hop<'found>>::from(edge.ends())?;
        (hop.takes() == from.takes()).then_some(Joined(edge)).filter(|_| hop.gives() == to.gives())
    }

    fn resolved(&self, callee: &Name, owner: CallOwner<'_>) -> Option<&'graph Edge<'found, 'file>> {
        let candidates: Vec<&'graph Edge<'found, 'file>> = self
            .named
            .get(callee)
            .into_iter()
            .flatten()
            .copied()
            .filter(|candidate| {
                owner.admits(candidate.transition().context().selfish().subject().map(AsRef::as_ref)).is_some()
            })
            .collect();
        Some(candidates).filter(|found| found.len() == 1).and_then(|found| found.first().copied())
    }

    fn stop(&self, edge: &Edge<'found, 'file>) -> Option<Stop> {
        Option::<Hop<'found>>::from(edge.ends()).and_then(|hop| self.hops.get(&hop)).copied()
    }
}
