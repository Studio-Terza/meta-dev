use std::borrow::Borrow;
use std::collections::BTreeMap;

use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateName;
use transition_facts::{Endpoint, Nature};

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
        let named = hint.and_then(|named| declaring.iter().find_map(|member| member.crate_named(named)));
        let found = named
            .or_else(|| declaring.iter().find_map(|member| member.in_crate(from)))
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
}
