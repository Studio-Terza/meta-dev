use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use newtype::Counter;
use transition_facts::Transition;

use crate::count::Count;

#[derive(Debug)]
pub(super) struct Hidden<'found>(&'found Transition);

impl<'found> Hidden<'found> {
    pub(super) fn new(found: &'found Transition) -> Hidden<'found> {
        Hidden(found)
    }
}

impl From<Hidden<'_>> for Violation {
    fn from(hidden: Hidden<'_>) -> Violation {
        Violation::new(AsRef::<Site>::as_ref(hidden.0).clone(), hidden)
    }
}

impl fmt::Display for Hidden<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let branches = self.0.branches();
        write!(
            formatter,
            "{} branches {branches} ways, {} unnamed intermediate states",
            self.0,
            branches.count().saturating_sub(1)
        )
    }
}

impl lint_layout::Finding for Hidden<'_> {
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
    pub(super) fn counted(written: &[&Transition], above: &[&Transition]) -> Tally {
        Tally {
            written: Count::new(written.len()),
            above: Count::new(above.len()),
            states: Count::total(above.iter().map(|found| Count::new(found.branches().count().saturating_sub(1)))),
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
