use std::collections::BTreeSet;
use std::fmt;
use std::num::NonZeroUsize;

use graph::{Function, Implementation, MethodShape, Placed, Site};
use lint_layout::Name;
use newtype::Wrapping;

use crate::query::catalog::{Catalog, Stocked};
use crate::query::kind::Kind;
use crate::query::term::Term;
use crate::role::Role;
use crate::score::Score;

#[derive(Clone, Debug)]
pub struct Offered<'model> {
    name: &'model Name,
    shape: MethodShape,
}

#[derive(Clone, Copy, Debug)]
enum Label<'model> {
    Named(&'model Name),
    Generic(&'model Implementation),
}

#[derive(Clone, Debug)]
pub struct Candidate<'model> {
    kind: Kind,
    krate: &'model Name,
    label: Label<'model>,
    site: Site<'model>,
    names: Vec<&'model Name>,
    methods: Vec<Offered<'model>>,
    related: Vec<&'model Name>,
}

impl<'model> From<&'model Function> for Offered<'model> {
    fn from(function: &'model Function) -> Offered<'model> {
        Offered { name: function.as_ref(), shape: MethodShape::from(function) }
    }
}

impl<'model> Candidate<'model> {
    const NAMED: Score = Score::new(3);
    const MENTIONED: Score = Score::new(1);
    const CLOSE: Score = Score::new(3);
    const BELOW_CLOSE: Score = Score::new(2);
    const SHOWN: usize = 5;

    #[must_use]
    pub(crate) fn declared(stocked: Stocked<'model>, kind: Kind, catalog: &Catalog<'model>) -> Candidate<'model> {
        let placed = stocked.placed();
        let id = placed.entity().declared();
        let implementers: BTreeSet<&Name> = catalog.implementers(id).collect();
        Candidate {
            kind,
            krate: placed.krate(),
            label: Label::Named(stocked.name()),
            site: placed.site(),
            names: vec![stocked.name()],
            methods: Catalog::OWNERS.iter().flat_map(|owner| catalog.owned_by(id, *owner)).map(Offered::from).collect(),
            related: implementers
                .into_iter()
                .chain(catalog.implemented_by(id).filter_map(Implementation::interface_name))
                .collect(),
        }
    }

    #[must_use]
    pub(crate) fn of_generic(
        placed: Placed<'model, &'model Implementation>,
        catalog: &Catalog<'model>,
    ) -> Candidate<'model> {
        let implementation = placed.entity();
        let implemented = Implementation::interface_name(implementation);
        Candidate {
            kind: Kind::GenericImpl,
            krate: placed.krate(),
            label: Label::Generic(implementation),
            site: placed.site(),
            names: Some(implementation.subject()).into_iter().chain(implemented).collect(),
            methods: catalog.seated(placed).map(Offered::from).collect(),
            related: Vec::new(),
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
    pub fn fitting(&self, role: &Role) -> Option<&Offered<'model>> {
        self.methods
            .iter()
            .map(|method| (role.agreement(&method.shape), method))
            .filter(|(agreement, _)| *agreement >= Self::CLOSE)
            .max_by_key(|(agreement, _)| *agreement)
            .map(|(_, method)| method)
    }

    #[must_use]
    pub fn shaped(&self, role: &Role) -> Score {
        self.fitting(role).map_or_else(Score::default, |method| {
            Score::new(role.agreement(&method.shape).into_inner().saturating_sub(Self::BELOW_CLOSE.into_inner()))
        })
    }

    fn mentions(&self) -> impl Iterator<Item = &'model Name> {
        self.methods.iter().map(|method| method.name).chain(self.related.iter().copied())
    }
}

impl Label<'_> {
    fn written(formatter: &mut fmt::Formatter<'_>, found: &Implementation) -> fmt::Result {
        Implementation::interface_name(found).map_or(Ok(()), |implemented| write!(formatter, "{implemented} for "))?;
        write!(formatter, "{}", found.subject())
    }
}

impl fmt::Display for Offered<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.name, self.shape)
    }
}

impl fmt::Display for Label<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Label::Named(name) => write!(formatter, "{name}"),
            Label::Generic(found) => Label::written(formatter, found),
        }
    }
}

impl fmt::Display for Candidate<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} in {} at {}", self.kind, self.label, self.krate, self.site)?;
        let shown: Vec<String> = self.related.iter().take(Self::SHOWN).map(ToString::to_string).collect();
        let more = self.related.len().saturating_sub(Self::SHOWN);
        shown.first().map_or(Ok(()), |_| write!(formatter, "; {} {}", self.kind.relation(), shown.join(", ")))?;
        NonZeroUsize::new(more).map_or(Ok(()), |hidden| write!(formatter, " and {hidden} more"))
    }
}
