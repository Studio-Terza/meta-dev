mod boundary;
mod candidate;
mod kind;
mod matches;
mod reply;
mod term;
mod top;

use std::collections::BTreeMap;
use std::path::PathBuf;

use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;
use lint_transition_graph::{Ask, Listed, TypeSystem};
use newtype::Wrapping;
use transition_facts::{Ground, Transitions};
use type_facts::Facts;
use workspace_metadata::Metadata;

use crate::boundary::CrateSizes;
use crate::role::Role;
use candidate::Candidate;
use kind::Kind;
use matches::Matches;
use reply::Reply;
pub use term::Term;
use top::Top;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    terms: Vec<Term>,
    roles: Vec<Role>,
    questions: Vec<Ask>,
    top: Option<Top>,
    roots: Vec<PathBuf>,
    crates: Option<()>,
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
        self.questions.push(ask);
        self
    }

    pub(crate) fn with_root(mut self, root: PathBuf) -> Query {
        self.roots.push(root);
        self
    }

    pub(crate) fn with_crates(mut self) -> Query {
        self.crates = Some(());
        self
    }

    pub(crate) fn with_top(mut self, top: Top) -> Query {
        self.top = Some(top);
        self
    }

    fn searches(&self) -> Option<()> {
        self.terms.first().map(|_| ()).or_else(|| self.roles.first().map(|_| ()))
    }

    pub(crate) fn asks(&self) -> Option<&Query> {
        self.questions
            .first()
            .map(|_| self)
            .or_else(|| self.searches().map(|()| self))
            .or_else(|| self.crates.map(|()| self))
    }

    #[must_use]
    pub fn reply<'view, 'found, 'file>(
        &'view self,
        system: &'view TypeSystem<'found, 'file>,
    ) -> Reply<'view, 'found, 'file> {
        let extraction = system.extraction();
        let matches = self.searches().map(|()| self.matches(extraction.facts(), extraction.transitions()));
        let listed = self.top.map(|top| Listed::new(top.into_inner()));
        Reply::new(matches, self.questions.iter().map(|ask| ask.answered(system.containment(), listed)).collect())
    }

    pub(crate) fn sized<'files>(
        &self,
        files: &[SourceFile],
        workspaces: &'files [Metadata],
    ) -> Option<CrateSizes<'files>> {
        self.crates.map(|()| CrateSizes::measured(files, workspaces))
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
        Matches::ranked(traits.chain(generics).chain(types), &self.terms, &self.roles, self.top.unwrap_or(Top::DEFAULT))
    }
}
