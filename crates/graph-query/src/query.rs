mod boundary;
mod candidate;
mod catalog;
mod kind;
mod matches;
mod reply;
mod term;
mod top;

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::path::PathBuf;

use graph::ItemKind;
use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;
use lint_transition_graph::{Ask, Ground, Listed, Transitions, TypeSystem};
use newtype::Wrapping;
use workspace_metadata::Metadata;

use crate::boundary::CrateSizes;
use crate::role::Role;
use candidate::Candidate;
use catalog::{Catalog, Stocked};
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
        let matches = self.searches().map(|()| self.matches(system.transitions()));
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
    pub fn matches<'view>(&'view self, transitions: &Transitions<'view>) -> Matches<'view> {
        let model = transitions.model();
        let grounds: BTreeMap<(&Name, &Name), Ground> = transitions
            .contexts()
            .iter()
            .filter_map(|context| {
                let krate: &Name = AsRef::<CrateName>::as_ref(context).borrow();
                context.subject().map(|subject| ((krate, subject.as_ref()), *context.as_ref()))
            })
            .collect();
        let ground = |stocked: Stocked<'_>| grounds.get(&(stocked.placed().krate(), stocked.name())).copied();
        let catalog = Catalog::from(model);
        let traits = catalog
            .declared(&[ItemKind::Trait])
            .map(|stocked| Candidate::declared(stocked, Kind::of_trait(ground(stocked)), &catalog));
        let generics = catalog.generics().map(|placed| Candidate::of_generic(placed, &catalog));
        let types = catalog.declared(&Catalog::TYPES).filter_map(|stocked| {
            Kind::of_trait(ground(stocked)).implementer().map(|kind| Candidate::declared(stocked, kind, &catalog))
        });
        Matches::ranked(traits.chain(generics).chain(types), &self.terms, &self.roles, self.top.unwrap_or(Top::DEFAULT))
    }
}
