use std::collections::BTreeMap;
use std::path::Path;

use graph::Site;

use crate::group::Group;
use crate::member::{Keyed, Nested};

#[derive(Debug)]
pub(super) struct Kept<'file, Member: Keyed> {
    groups: Vec<Group<Member>>,
    homes: BTreeMap<&'file Path, Vec<usize>>,
}

impl<'file, Member: Keyed + Nested + AsRef<Site<'file>>> Kept<'file, Member> {
    pub(super) fn into_groups(self) -> Vec<Group<Member>> {
        self.groups
    }

    fn keeping(self, group: Group<Member>) -> Kept<'file, Member> {
        let at = self.groups.len();
        let homes = group.members().iter().map(|member| member.as_ref().path()).fold(self.homes, |mut homes, path| {
            let placed = homes.entry(path).or_default();
            placed.extend(placed.last().filter(|last| **last == at).map_or(Some(at), |_| None));
            homes
        });
        let mut groups = self.groups;
        groups.push(group);
        Kept { groups, homes }
    }

    fn cover(&self, member: &Member) -> Option<&Member> {
        let near = self.homes.get(member.as_ref().path()).into_iter().flatten();
        near.filter_map(|at| self.groups.get(*at)).flat_map(Group::members).find_map(|outer| member.lies_within(outer))
    }
}

impl<'file, Member: Keyed + Nested + AsRef<Site<'file>>> FromIterator<Group<Member>> for Kept<'file, Member> {
    fn from_iter<Ranked: IntoIterator<Item = Group<Member>>>(ranked: Ranked) -> Kept<'file, Member> {
        let empty = Kept { groups: Vec::new(), homes: BTreeMap::new() };
        ranked.into_iter().fold(empty, |kept, group| {
            let opened = group.members().iter().any(|member| kept.cover(member).is_none()).then_some(group);
            opened.into_iter().fold(kept, Kept::keeping)
        })
    }
}
