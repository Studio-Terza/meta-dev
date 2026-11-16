mod boundary;

use std::cmp::Reverse;
use std::collections::BTreeMap;

use graph::Site;

use crate::group::Group;
use crate::member::{Keyed, Nested};
use boundary::Kept;

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
}

impl<Member: Keyed + Nested> Groups<Member> {
    #[must_use]
    pub fn outermost<'file, Rank: Ord>(mut self, rank: impl Fn(&Member::Key) -> Rank) -> Groups<Member>
    where
        Member: AsRef<Site<'file>>,
    {
        self.0.sort_by_key(|group| Reverse(rank(group.key())));
        Groups(self.0.into_iter().collect::<Kept<'file, Member>>().into_groups())
    }
}
