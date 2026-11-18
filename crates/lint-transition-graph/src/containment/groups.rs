use lint_layout::Name;
use std::collections::BTreeSet;
use std::fmt;

use graph::{Event, Trees};

use crate::count::Count;

use super::Containment;
use super::link::{Link, Reversed};
use super::member::Member;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Groups<'found>(Vec<Vec<Member<'found>>>);

impl<'found> Groups<'found> {
    pub(crate) fn linked(
        members: &BTreeSet<Member<'found>>,
        directed: impl Iterator<Item = Link<'found>>,
    ) -> Groups<'found> {
        let links = Containment::adjacency(directed.flat_map(|link| [link, Link::from(Reversed::from(link))]));
        let trees: Trees<Member<'found>> = Event::walked(members.iter().copied(), |member: Member<'found>| {
            links.get(&member).into_iter().flatten().copied()
        })
        .collect();
        let mut groups: Vec<Vec<Member<'found>>> = trees
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
                let names: Vec<String> = group.iter().map(|member| <&Name>::from(*member).to_string()).collect();
                format!("[{}]", names.join(" "))
            })
            .collect();
        write!(formatter, "{} groups: {}", Count::new(self.0.len()), groups.join(" "))
    }
}
