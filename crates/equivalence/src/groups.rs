use std::cmp::Reverse;
use std::collections::BTreeMap;

use crate::group::Group;
use crate::member::{Keyed, Nested};

#[derive(Debug)]
pub struct Groups<Member: Keyed>(Vec<Group<Member>>);

impl<Member: Keyed> Groups<Member> {
    pub fn grouped(found: impl IntoIterator<Item = Member>) -> Groups<Member> {
        let keyed = found.into_iter().fold(BTreeMap::<Member::Key, Vec<Member>>::new(), |mut groups, member| {
            groups.entry(member.key()).or_default().push(member);
            groups
        });
        Groups(
            keyed
                .into_iter()
                .filter(|(_, members)| members.len() > 1)
                .map(|(key, members)| Group::new(key, members))
                .collect(),
        )
    }

    #[must_use]
    pub fn listed(self) -> Vec<Group<Member>> {
        self.0
    }

    #[must_use]
    pub fn ranked<Rank: Ord>(mut self, rank: impl Fn(&Member::Key) -> Rank) -> Groups<Member> {
        self.0.sort_by_key(|group| Reverse(rank(group.key())));
        self
    }
}

impl<Member: Keyed + Nested> Groups<Member> {
    #[must_use]
    pub fn maximal(self) -> Groups<Member> {
        Groups(self.0.into_iter().fold(Vec::new(), Self::admitted))
    }

    fn admitted(mut kept: Vec<Group<Member>>, group: Group<Member>) -> Vec<Group<Member>> {
        kept.extend(group.members().iter().any(|member| Self::cover(&kept, member).is_none()).then_some(group));
        kept
    }

    fn cover<'kept>(kept: &'kept [Group<Member>], member: &Member) -> Option<&'kept Member> {
        kept.iter().flat_map(Group::members).find_map(|outer| member.lies_within(outer))
    }
}
