mod boundary;
mod candidate;
mod kind;
mod matches;
mod reply;
mod term;
mod top;

use std::collections::BTreeMap;
use std::path::PathBuf;

use lint_layout::Name;
use lint_placement::CrateName;
use lint_transition_graph::{Ask, TypeSystem};
use transition_facts::{Ground, Transitions};
use type_facts::Facts;

use crate::role::Role;
use candidate::Candidate;
use kind::Kind;
use matches::Matches;
use reply::Reply;
pub use term::Term;
use top::Top;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    terms: Vec<Term>,
    roles: Vec<Role>,
    asks: Vec<Ask>,
    top: Top,
    roots: Vec<PathBuf>,
}

impl Default for Query {
    fn default() -> Query {
        Query { terms: Vec::new(), roles: Vec::new(), asks: Vec::new(), top: Top::DEFAULT, roots: Vec::new() }
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

    pub(crate) fn with_ask(mut self, ask: Ask) -> Query {
        self.asks.push(ask);
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
        let searches = self.terms.first().map(|_| self).or_else(|| self.roles.first().map(|_| self));
        self.asks.first().map(|_| self).or(searches)
    }

    #[must_use]
    pub fn reply<'view, 'found, 'file>(
        &'view self,
        system: &'view TypeSystem<'found, 'file>,
    ) -> Reply<'view, 'found, 'file> {
        let extraction = system.extraction();
        let searched = self.terms.first().map(|_| ()).or_else(|| self.roles.first().map(|_| ()));
        let matches = searched.map(|()| self.matches(extraction.facts(), extraction.transitions()));
        Reply::new(matches, self.asks.iter().map(|ask| ask.answered(system.containment())).collect())
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
