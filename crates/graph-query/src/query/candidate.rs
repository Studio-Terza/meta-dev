use std::collections::BTreeSet;
use std::fmt;
use std::num::NonZeroUsize;

use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use newtype::Newtype;
use type_facts::{Facts, GenericImpl, Implementation, Method, TraitFact, TypeFact};

use crate::query::kind::Kind;
use crate::role::Role;
use crate::score::Score;
use crate::term::Term;

#[derive(Clone, Copy, Debug)]
enum Label<'facts, 'file> {
    Named(&'facts Name),
    Generic(&'facts GenericImpl<'file>),
}

#[derive(Clone, Debug)]
pub struct Candidate<'facts, 'file> {
    kind: Kind,
    krate: &'facts CrateName,
    label: Label<'facts, 'file>,
    site: &'facts Site<'file>,
    names: Vec<&'facts Name>,
    methods: Vec<&'facts Method>,
    related: Vec<&'facts Name>,
}

impl<'facts, 'file> Candidate<'facts, 'file> {
    const NAMED: Score = Score::new(3);
    const MENTIONED: Score = Score::new(1);
    const CLOSE: Score = Score::new(3);
    const BELOW_CLOSE: Score = Score::new(2);
    const SHOWN: usize = 5;

    #[must_use]
    pub fn of_trait(
        found: &'facts TraitFact<'file>,
        kind: Kind,
        facts: &'facts Facts<'file>,
    ) -> Candidate<'facts, 'file> {
        let name: &Name = found.as_ref();
        let implementers: BTreeSet<&Name> = facts.implementers(name).into_iter().collect();
        Candidate {
            kind,
            krate: found.as_ref(),
            label: Label::Named(name),
            site: found.as_ref(),
            names: vec![name],
            methods: AsRef::<[Method]>::as_ref(found).iter().collect(),
            related: implementers.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn of_generic(found: &'facts GenericImpl<'file>) -> Candidate<'facts, 'file> {
        Candidate {
            kind: Kind::GenericImpl,
            krate: found.as_ref(),
            label: Label::Generic(found),
            site: found.as_ref(),
            names: Some(found.as_ref()).into_iter().chain(found.implemented()).collect(),
            methods: AsRef::<[Method]>::as_ref(found).iter().collect(),
            related: Vec::new(),
        }
    }

    #[must_use]
    pub fn of_type(
        found: &'facts TypeFact<'file>,
        kind: Kind,
        facts: &'facts Facts<'file>,
    ) -> Candidate<'facts, 'file> {
        let implementations = facts.implementations(found);
        Candidate {
            kind,
            krate: found.as_ref(),
            label: Label::Named(found.as_ref()),
            site: found.as_ref(),
            names: vec![found.as_ref()],
            methods: facts
                .methods(found)
                .iter()
                .chain(implementations.iter().flat_map(Implementation::methods))
                .collect(),
            related: implementations.iter().map(AsRef::as_ref).collect(),
        }
    }

    #[must_use]
    pub fn named(&self, terms: &[Term]) -> Score {
        terms
            .iter()
            .map(|term| {
                let named = self.names.iter().find_map(|name| term.found_in(name)).map(|_| Self::NAMED);
                let mentioned = self.mentions().find_map(|name| term.found_in(name)).map(|_| Self::MENTIONED);
                named.or(mentioned).unwrap_or_default()
            })
            .fold(Score::default(), |total, each| total + each)
    }

    #[must_use]
    pub fn fitting(&self, role: &Role) -> Option<&'facts Method> {
        self.methods
            .iter()
            .copied()
            .map(|method| (role.agreement(method.shape()), method))
            .filter(|(agreement, _)| *agreement >= Self::CLOSE)
            .max_by_key(|(agreement, _)| *agreement)
            .map(|(_, method)| method)
    }

    #[must_use]
    pub fn shaped(&self, role: &Role) -> Score {
        self.fitting(role).map_or_else(Score::default, |method| {
            Score::new(role.agreement(method.shape()).into_inner().saturating_sub(Self::BELOW_CLOSE.into_inner()))
        })
    }

    fn mentions(&self) -> impl Iterator<Item = &'facts Name> {
        self.methods.iter().map(|method| AsRef::<Name>::as_ref(*method)).chain(self.related.iter().copied())
    }
}

impl fmt::Display for Label<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Label::Named(name) => write!(formatter, "{name}"),
            Label::Generic(found) => write!(formatter, "{found}"),
        }
    }
}

impl fmt::Display for Candidate<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} in {} at {}", self.kind, self.label, self.krate, self.site)?;
        let shown: Vec<String> = self.related.iter().take(Self::SHOWN).map(ToString::to_string).collect();
        let more = self.related.len().saturating_sub(Self::SHOWN);
        shown.first().map_or(Ok(()), |_| write!(formatter, "; {} {}", self.kind.relation(), shown.join(", ")))?;
        NonZeroUsize::new(more).map_or(Ok(()), |more| write!(formatter, " and {more} more"))
    }
}
