use std::collections::BTreeMap;

use lint_layout::Name;
use newtype::{Arbitrary, Newtype};

use super::walk::Walk;
use crate::edge::Edge;
use crate::end::{End, Hop};
use transition_facts::{Owner, Wrapping};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Stop(usize);

#[derive(Debug)]
pub(super) struct Stops<'graph, 'found, 'file> {
    named: BTreeMap<&'graph Name, Vec<&'graph Edge<'found, 'file>>>,
    hops: BTreeMap<Hop<'found>, Stop>,
    joins: Vec<&'graph Edge<'found, 'file>>,
}

impl<'graph, 'found, 'file> Stops<'graph, 'found, 'file> {
    pub(super) fn indexed(edges: &'graph [Edge<'found, 'file>]) -> Stops<'graph, 'found, 'file> {
        let joins: Vec<&'graph Edge<'found, 'file>> =
            edges.iter().filter(|edge| edge.form().written().is_some()).collect();
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
        let walker = edge.transition().context().subject().map(AsRef::<Name>::as_ref);
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
        let stops = called.iter().filter_map(|called| self.stop(called)).collect();
        Walk::new(edge, called, stops)
    }

    pub(super) fn joining(
        &self,
        first: &Edge<'found, 'file>,
        last: &Edge<'found, 'file>,
    ) -> Option<&'graph Edge<'found, 'file>> {
        let ends = Option::<Hop<'found>>::from(first.ends()).zip(Option::<Hop<'found>>::from(last.ends()));
        ends.and_then(|(from, to)| self.joins.iter().copied().find(|edge| Self::closes(edge, &from, &to).is_some()))
    }

    fn output(edge: &'graph Edge<'found, 'file>) -> Option<&'graph Name> {
        (edge.transition().function().wrapping() == Wrapping::Plain)
            .then(|| AsRef::<End<'found>>::as_ref(edge.ends()).as_ref())
    }

    fn closes(edge: &Edge<'found, 'file>, from: &Hop<'found>, to: &Hop<'found>) -> Option<Hop<'found>> {
        Option::<Hop<'found>>::from(edge.ends()).filter(|hop| hop.joins(from, to).is_some())
    }

    fn resolved(&self, callee: &Name, owner: Owner<'_>) -> Option<&'graph Edge<'found, 'file>> {
        let candidates: Vec<&'graph Edge<'found, 'file>> = self
            .named
            .get(callee)
            .into_iter()
            .flatten()
            .copied()
            .filter(|candidate| owner.admits(candidate.transition().context().subject().map(AsRef::as_ref)).is_some())
            .collect();
        Some(candidates).filter(|found| found.len() == 1).and_then(|found| found.first().copied())
    }

    fn stop(&self, edge: &Edge<'found, 'file>) -> Option<Stop> {
        Option::<Hop<'found>>::from(edge.ends()).and_then(|hop| self.hops.get(&hop)).copied()
    }
}
