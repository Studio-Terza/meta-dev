mod boundary;
mod candidate;
mod kind;
mod matches;
mod term;
mod top;

use std::collections::BTreeMap;
use std::path::PathBuf;

use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::{Ground, Transitions};
use type_facts::Facts;

use crate::role::Role;
use candidate::Candidate;
use kind::Kind;
use matches::Matches;
pub use term::Term;
use top::Top;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    terms: Vec<Term>,
    roles: Vec<Role>,
    top: Top,
    roots: Vec<PathBuf>,
}

impl Default for Query {
    fn default() -> Query {
        Query { terms: Vec::new(), roles: Vec::new(), top: Top::DEFAULT, roots: Vec::new() }
    }
}

impl Query {
    pub(crate) fn with_term(mut self, term: Term) -> Query {
        self.terms.push(term);
        self
    }

    pub(crate) fn with_role(mut self, role: Role) -> Query {
        self.roles.push(role);
        self
    }

    pub(crate) fn with_root(mut self, root: PathBuf) -> Query {
        self.roots.push(root);
        self
    }

    pub(crate) fn with_top(mut self, top: Top) -> Query {
        self.top = top;
        self
    }

    pub(crate) fn asks(&self) -> Option<&Query> {
        (!(self.terms.is_empty() && self.roles.is_empty())).then_some(self)
    }

    #[must_use]
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    #[must_use]
    pub fn matches<'facts, 'file>(
        &'facts self,
        facts: &'facts Facts<'file>,
        transitions: &Transitions<'file>,
    ) -> Matches<'facts, 'file> {
        let grounds: BTreeMap<(&CrateName, &Name), Ground> = transitions
            .contexts()
            .iter()
            .filter_map(|context| {
                context.subject().map(|subject| ((context.as_ref(), subject.as_ref()), *context.as_ref()))
            })
            .collect();
        let ground = |krate: &CrateName, name: &Name| grounds.get(&(krate, name)).copied();
        let traits = facts
            .traits()
            .iter()
            .map(|found| Candidate::of_trait(found, Kind::of_trait(ground(found.as_ref(), found.as_ref())), facts));
        let generics = facts.generic_impls().iter().map(Candidate::of_generic);
        let types = facts.types().iter().filter_map(|found| {
            Kind::of_trait(ground(found.as_ref(), found.as_ref()))
                .implementer()
                .map(|kind| Candidate::of_type(found, kind, facts))
        });
        Matches::ranked(traits.chain(generics).chain(types), &self.terms, &self.roles, self.top)
    }
}
