use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use newtype::Counter;
use transition_facts::Transition;

use crate::count::Count;

#[derive(Debug)]
pub(super) struct Hidden<'found, 'file>(&'found Transition<'found, 'file>);

impl<'found, 'file> Hidden<'found, 'file> {
    pub(super) fn new(found: &'found Transition<'found, 'file>) -> Hidden<'found, 'file> {
        Hidden(found)
    }
}

impl<'file> From<Hidden<'_, 'file>> for Violation<'file> {
    fn from(hidden: Hidden<'_, 'file>) -> Violation<'file> {
        Violation::new(*AsRef::<Site<'file>>::as_ref(hidden.0), hidden)
    }
}

impl fmt::Display for Hidden<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let branches = self.0.function().branches();
        write!(
            formatter,
            "{} branches {branches} ways, {} unnamed intermediate states",
            self.0,
            branches.count().saturating_sub(1)
        )
    }
}

impl lint_layout::Finding for Hidden<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "name the intermediate states as an enum or a typestate and split the body into small edges between them",
        )
    }
}

#[derive(Debug)]
pub(super) struct Tally {
    written: Count,
    above: Count,
    states: Count,
}

impl Tally {
    pub(super) fn counted(written: &[&Transition<'_, '_>], above: &[&Transition<'_, '_>]) -> Tally {
        Tally {
            written: Count::new(written.len()),
            above: Count::new(above.len()),
            states: Count::total(
                above.iter().map(|found| Count::new(found.function().branches().count().saturating_sub(1))),
            ),
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {} written edges, {} branch six ways or more, holding {} unnamed intermediate states",
            self.written, self.above, self.states
        )
    }
}

impl lint_layout::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new(
            "name the intermediate states of the edges listed as enums or typestates, with small edges between them",
        )
    }
}
