use std::collections::BTreeMap;
use std::fmt;

use lint_core::Fix;
use lint_core::{Site, Violation};
use lint_layout::{Name, Whole, Word, words};
use transition_facts::Transition;

use crate::count::Count;
use crate::edge::{Edge, Role};
use crate::end::Key;

#[derive(Debug)]
pub(super) struct Family<'found, 'file> {
    key: Key<'found>,
    members: Vec<&'found Transition<'found, 'file>>,
}

impl<'found, 'file> Family<'found, 'file> {
    const CONVERSIONS: [Word<'static>; 3] = words!["from", "try_from", "from_str"];

    pub(super) fn repeated_among(edges: &'found [Edge<'found, 'file>]) -> Vec<Family<'found, 'file>> {
        let keyed = edges.iter().filter(|edge| edge.role() == Role::Written).fold(
            BTreeMap::new(),
            |mut keys: BTreeMap<Key<'found>, Vec<&'found Transition<'found, 'file>>>, edge| {
                edge.ends().key().into_iter().for_each(|key| keys.entry(key).or_default().push(edge.transition()));
                keys
            },
        );
        keyed
            .into_iter()
            .filter(|(key, members)| members.len() > 1 && key.algebraic().is_none())
            .map(|(key, members)| Family { key, members })
            .collect()
    }

    pub(super) fn violation(&self) -> Option<Violation<'file>> {
        let first = self.members.iter().map(|member| *AsRef::<Site<'file>>::as_ref(*member)).min();
        first.map(|site| Violation::new(site, self))
    }

    pub(super) fn size(&self) -> Count {
        Count::new(self.members.len())
    }

    fn canonical(&self) -> Option<&&'found Transition<'found, 'file>> {
        self.members
            .iter()
            .find(|member| AsRef::<Name>::as_ref(member.function()).matches::<Whole>(&Self::CONVERSIONS).is_some())
    }
}

impl fmt::Display for Family<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let paths: Vec<String> =
            self.members.iter().map(|member| format!("{member} at {}", AsRef::<Site>::as_ref(*member))).collect();
        write!(formatter, "{} has {} transitions: {}", self.key, self.members.len(), paths.join(", "))?;
        match self.canonical() {
            Some(conversion) => write!(formatter, "; {conversion} is the standard conversion"),
            None => Ok(()),
        }
    }
}

impl lint_core::Finding for Family<'_, '_> {
    fn fix(&self) -> Fix {
        match self.canonical() {
            Some(_) => Fix::new(
                "keep the standard conversion and route the other transitions through it; where they mean different things, give each its own output type",
            ),
            None => Fix::new(
                "keep one transition, as From, TryFrom or FromStr where it fits, and route the others through it; delete a hand-written copy of a map, filter or fold over the element transition; where the transitions mean different things, give each its own output type",
            ),
        }
    }
}
