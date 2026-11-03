use std::collections::BTreeMap;

use body_lifts::Body;
use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::{Ground, Mutation, Transition};

use crate::end::Ends;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Written,
    Lift,
    Operation,
}

impl From<Body> for Role {
    fn from(body: Body) -> Role {
        match body {
            Body::Written => Role::Written,
            Body::Mapped | Body::Focused | Body::Routed => Role::Lift,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Edge<'found, 'file> {
    transition: Transition<'found, 'file>,
    ends: Ends<'found>,
    fixed_by: Option<&'found Name>,
}

impl<'found, 'file> Edge<'found, 'file> {
    pub(crate) fn new(
        transition: Transition<'found, 'file>,
        ends: Ends<'found>,
        fixed_by: Option<&'found Name>,
    ) -> Edge<'found, 'file> {
        Edge { transition, ends, fixed_by }
    }

    pub(crate) fn role(&self) -> Role {
        (*AsRef::<Ground>::as_ref(&self.transition) == Ground::Store)
            .then_some(Role::Operation)
            .filter(|_| self.transition.function().mutation() == Mutation::ThroughSelf)
            .unwrap_or_else(|| Role::from(self.transition.function().body()))
    }

    pub(crate) fn ends(&self) -> &Ends<'found> {
        &self.ends
    }

    pub(crate) fn transition(&self) -> &Transition<'found, 'file> {
        &self.transition
    }

    pub(crate) fn site(&self) -> Site<'file> {
        *AsRef::<Site<'file>>::as_ref(&self.transition)
    }

    pub(crate) fn fixed_by(&self) -> Option<&'found Name> {
        self.fixed_by
    }

    pub(crate) fn by_crate<'edge>(
        edges: impl Iterator<Item = &'edge Edge<'found, 'file>>,
    ) -> BTreeMap<&'edge CrateName, Vec<&'edge Edge<'found, 'file>>> {
        edges.fold(BTreeMap::new(), |mut crates, edge| {
            crates.entry(edge.transition().as_ref()).or_default().push(edge);
            crates
        })
    }
}
