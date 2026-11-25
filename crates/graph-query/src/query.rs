mod boundary;
mod candidate;
mod catalog;
mod consumer;
mod kind;
mod matches;
mod reply;
mod term;
mod top;

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use algebra::{Monoid, Semigroup};
use graph::{ItemKind, Model};
use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;
use lint_transition_graph::{Ask, Ground, Listed, Transitions, TypeSystem};
use newtype::Wrapping;
use newtype::proptest::arbitrary::any;
use newtype::proptest::collection::vec;
use newtype::proptest::strategy::Strategy;
use workspace_metadata::Metadata;

use crate::boundary::CrateSizes;
use crate::role::Role;
use candidate::Candidate;
use catalog::{Catalog, Stocked};
pub(crate) use consumer::Consumer;
use kind::Kind;
use matches::Matches;
use reply::Reply;
pub use term::Term;
use top::Top;

#[derive(newtype::Arbitrary, Clone, Debug, Default, PartialEq, Eq)]
#[newtype(generate = Query::sampled)]
pub struct Query {
    terms: Vec<Term>,
    roles: Vec<Role>,
    questions: Vec<Ask>,
    top: Option<Top>,
    roots: Vec<PathBuf>,
    crates: Option<()>,
    consumers: Vec<Consumer>,
}

impl Semigroup for Query {
    fn combine(self, later: Query) -> Query {
        Query {
            terms: self.terms.combine(later.terms),
            roles: self.roles.combine(later.roles),
            questions: self.questions.combine(later.questions),
            top: later.top.or(self.top),
            roots: self.roots.combine(later.roots),
            crates: self.crates.or(later.crates),
            consumers: self.consumers.combine(later.consumers),
        }
    }
}

impl Monoid for Query {}

impl From<Term> for Query {
    fn from(term: Term) -> Query {
        Query { terms: vec![term], ..Query::default() }
    }
}

impl From<Role> for Query {
    fn from(role: Role) -> Query {
        Query { roles: vec![role], ..Query::default() }
    }
}

impl From<Ask> for Query {
    fn from(ask: Ask) -> Query {
        Query { questions: vec![ask], ..Query::default() }
    }
}

impl From<PathBuf> for Query {
    fn from(root: PathBuf) -> Query {
        Query { roots: vec![root], ..Query::default() }
    }
}

impl From<Consumer> for Query {
    fn from(consumer: Consumer) -> Query {
        Query { consumers: vec![consumer], ..Query::default() }
    }
}

impl From<Top> for Query {
    fn from(top: Top) -> Query {
        Query { top: Some(top), ..Query::default() }
    }
}

impl Query {
    pub(crate) const CRATES: Query = Query {
        terms: Vec::new(),
        roles: Vec::new(),
        questions: Vec::new(),
        top: None,
        roots: Vec::new(),
        crates: Some(()),
        consumers: Vec::new(),
    };

    const SAMPLED: Range<usize> = 0..4;

    fn sampled() -> impl Strategy<Value = Query> {
        Strategy::prop_map(
            (
                vec(any::<Term>(), Self::SAMPLED),
                vec(any::<Role>(), Self::SAMPLED),
                vec(any::<Ask>(), Self::SAMPLED),
                any::<Option<Top>>(),
                vec(any::<PathBuf>(), Self::SAMPLED),
                any::<Option<()>>(),
                vec(any::<Consumer>(), Self::SAMPLED),
            ),
            |(terms, roles, questions, top, roots, crates, consumers)| Query {
                terms,
                roles,
                questions,
                top,
                roots,
                crates,
                consumers,
            },
        )
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
            .or_else(|| self.consumers.first().map(|_| self))
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

    pub(crate) fn consumed<'model>(&self, model: &'model Model) -> BTreeSet<&'model Path> {
        let files: BTreeSet<&Path> = self.consumers.iter().map(AsRef::as_ref).collect();
        files.first().map_or_default(|_| model.consumed_by(&files))
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
                context.selfish().subject().map(|subject| ((krate, subject.as_ref()), *context.as_ref()))
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
