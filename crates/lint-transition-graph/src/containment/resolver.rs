use std::borrow::Borrow;
use std::collections::BTreeMap;

use lint_item_graph::{ItemGraph, NodeIndex};
use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateName;
use transition_facts::{Endpoint, Nature, Transition};

use super::link::Link;
use super::member::Member;

#[derive(Debug, Default)]
pub(crate) struct Resolver<'found> {
    declaring: BTreeMap<&'found Name, Vec<Member<'found>>>,
}

impl<'found> Resolver<'found> {
    const EXTERNAL: [Word<'static>; 3] = words!["std", "core", "alloc"];

    pub(crate) fn of_members(members: impl Iterator<Item = Member<'found>>) -> Resolver<'found> {
        Resolver {
            declaring: members.fold(BTreeMap::new(), |mut declaring: BTreeMap<_, Vec<_>>, member| {
                declaring.entry(member.name()).or_default().push(member);
                declaring
            }),
        }
    }

    pub(crate) fn member(&self, from: &CrateName, name: &Name, hint: Option<&Name>) -> Option<Member<'found>> {
        let declaring = self.declaring.get(name)?;
        let external = hint.and_then(|named| named.matches::<Whole>(&Self::EXTERNAL));
        let krate = |member: &&Member<'found>| member.cluster().krate();
        let named = hint
            .and_then(|named| declaring.iter().find(|member| krate(member).map(Borrow::<Name>::borrow) == Some(named)));
        let found = named
            .or_else(|| declaring.iter().find(|member| krate(member) == Some(from)))
            .copied()
            .or_else(|| declaring.first().copied().filter(|_| declaring.len() == 1));
        external.map_or(found, |_| None)
    }

    pub(crate) fn endpoint(&self, from: &CrateName, endpoint: &Endpoint) -> Option<Member<'found>> {
        match endpoint.nature() {
            Nature::Named => self.member(from, endpoint.as_ref(), endpoint.origin().map(Borrow::borrow)),
            Nature::Interface => self.member(from, endpoint.as_ref(), None),
            Nature::Primitive | Nature::Boxed => None,
        }
    }

    pub(crate) fn bodies(&self, krate: &CrateName, graph: &ItemGraph) -> Vec<Link<'found>> {
        let owners = graph.links().iter().enumerate();
        owners.flat_map(|(index, used)| self.body(krate, graph, NodeIndex::from(index), used)).collect()
    }

    fn body(&self, krate: &CrateName, graph: &ItemGraph, owner: NodeIndex, used: &[NodeIndex]) -> Vec<Link<'found>> {
        let local = |index: NodeIndex| {
            let name = graph.name_at(index)?;
            self.member(krate, name, None).filter(|member| member.cluster().krate() == Some(krate))
        };
        let holder = local(owner);
        used.iter()
            .filter_map(|index| local(*index))
            .filter_map(|end| holder.map(|holder| Link::new(holder, end)))
            .collect()
    }

    pub(crate) fn signature(&self, transition: Transition<'found, '_>) -> Vec<Link<'found>> {
        let from: &CrateName = transition.context().as_ref();
        let holder = transition.context().subject().and_then(|subject| self.endpoint(from, subject));
        let ends = transition.takes().iter().chain([transition.gives().endpoint()]);
        ends.filter_map(|end| self.endpoint(from, end))
            .filter_map(|end| holder.map(|holder| Link::new(holder, end)))
            .collect()
    }
}
