use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use lint_item_graph::Traversal;

use crate::count::Count;

use super::member::Member;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Groups<'found>(Vec<Vec<Member<'found>>>);

impl<'found> Groups<'found> {
    pub(crate) fn linked(
        members: &BTreeSet<Member<'found>>,
        pairs: impl Iterator<Item = (Member<'found>, Member<'found>)>,
    ) -> Groups<'found> {
        let links = pairs.flat_map(|(one, other)| [(one, other), (other, one)]).fold(
            BTreeMap::new(),
            |mut links: BTreeMap<Member<'found>, BTreeSet<Member<'found>>>, (from, to)| {
                links.entry(from).or_default().insert(to);
                links
            },
        );
        let mut groups: Vec<Vec<Member<'found>>> =
            Traversal::trees(members.iter().copied(), |member: Member<'found>| {
                links.get(&member).into_iter().flatten().copied()
            })
            .into_iter()
            .map(|group| group.into_iter().collect::<BTreeSet<Member<'found>>>().into_iter().collect())
            .collect();
        groups.sort();
        Groups(groups)
    }

    pub(crate) fn split(self) -> Option<Groups<'found>> {
        Some(self).filter(|groups| groups.0.len() > 1)
    }
}

impl fmt::Display for Groups<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let groups: Vec<String> = self
            .0
            .iter()
            .map(|group| {
                let names: Vec<String> = group.iter().map(|member| member.name().to_string()).collect();
                format!("[{}]", names.join(" "))
            })
            .collect();
        write!(formatter, "{} groups: {}", Count::new(self.0.len()), groups.join(" "))
    }
}
