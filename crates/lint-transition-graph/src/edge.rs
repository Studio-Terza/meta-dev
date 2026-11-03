use body_lifts::Body;
use lint_layout::Name;
use transition_facts::{Ground, Mutation, Transition};

use crate::end::Ends;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Written,
    Lift,
    Operation,
}

#[derive(Debug)]
pub(crate) struct Edge<'found> {
    transition: &'found Transition,
    ends: Ends,
    fixed_by: Option<&'found Name>,
}

impl<'found> Edge<'found> {
    pub(crate) fn new(transition: &'found Transition, ends: Ends, fixed_by: Option<&'found Name>) -> Edge<'found> {
        Edge { transition, ends, fixed_by }
    }

    pub(crate) fn role(&self) -> Role {
        match (*AsRef::<Ground>::as_ref(self.transition), self.transition.mutation(), self.transition.body()) {
            (Ground::Store, Mutation::ThroughSelf, _) => Role::Operation,
            (_, _, Body::Written) => Role::Written,
            (_, _, Body::Mapped | Body::Focused | Body::Routed) => Role::Lift,
        }
    }

    pub(crate) fn ends(&self) -> &Ends {
        &self.ends
    }

    pub(crate) fn transition(&self) -> &'found Transition {
        self.transition
    }

    pub(crate) fn fixed_by(&self) -> Option<&'found Name> {
        self.fixed_by
    }
}
