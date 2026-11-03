use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::{Fix, Name, Whole, Word, words};
use type_facts::Transition;

use crate::count::Count;
use crate::end::Key;

#[derive(Debug)]
pub(super) struct Family<'found> {
    key: Key,
    members: Vec<&'found Transition>,
}

impl<'found> Family<'found> {
    const CONVERSIONS: [Word<'static>; 3] = words!["from", "try_from", "from_str"];

    pub(super) fn repeated(key: Key, members: Vec<&'found Transition>) -> Option<Family<'found>> {
        (members.len() > 1).then_some(Family { key, members })
    }

    pub(super) fn violation(&self) -> Option<Violation> {
        let first = self.members.iter().map(|member| AsRef::<Site>::as_ref(*member)).min();
        first.map(|site| Violation::new(site.clone(), self))
    }

    pub(super) fn size(&self) -> Count {
        Count::new(self.members.len())
    }

    fn canonical(&self) -> Option<&&'found Transition> {
        self.members
            .iter()
            .find(|member| AsRef::<Name>::as_ref(**member).matches::<Whole>(&Self::CONVERSIONS).is_some())
    }
}

impl fmt::Display for Family<'_> {
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

impl lint_layout::Finding for Family<'_> {
    fn fix(&self) -> Fix {
        match self.canonical() {
            Some(_) => Fix::new("keep the standard conversion and route the other transitions through it"),
            None => Fix::new(
                "keep one transition, as From, TryFrom or FromStr where it fits, and route the others through it",
            ),
        }
    }
}
