use std::cmp::Reverse;
use std::fmt;

use lint_layout::Name;
use newtype::Newtype;
use type_facts::{Method, MethodShape};

use crate::query::candidate::Candidate;
use crate::query::term::Term;
use crate::query::top::Top;
use crate::role::Role;
use crate::score::Score;

#[derive(Clone, Copy, Debug)]
struct Fit<'facts> {
    role: &'facts Role,
    method: &'facts Method,
}

#[derive(Clone, Debug)]
struct Ranked<'facts, 'file> {
    score: Score,
    candidate: Candidate<'facts, 'file>,
    fits: Vec<Fit<'facts>>,
}

#[derive(Clone, Debug)]
pub struct Matches<'facts, 'file>(Vec<Ranked<'facts, 'file>>);

impl<'facts, 'file> Ranked<'facts, 'file> {
    fn scored(candidate: Candidate<'facts, 'file>, terms: &[Term], roles: &'facts [Role]) -> Ranked<'facts, 'file> {
        let shaped = roles.iter().map(|role| candidate.shaped(role)).fold(Score::default(), |sum, each| sum + each);
        let fits = roles.iter().filter_map(|role| candidate.fitting(role).map(|method| Fit { role, method })).collect();
        Ranked { score: candidate.named(terms) + shaped, candidate, fits }
    }
}

impl<'facts, 'file> Matches<'facts, 'file> {
    #[must_use]
    pub fn ranked(
        candidates: impl Iterator<Item = Candidate<'facts, 'file>>,
        terms: &[Term],
        roles: &'facts [Role],
        top: Top,
    ) -> Matches<'facts, 'file> {
        let mut ranked: Vec<Ranked<'facts, 'file>> = candidates
            .map(|candidate| Ranked::scored(candidate, terms, roles))
            .filter(|ranked| ranked.score > Score::default())
            .collect();
        ranked.sort_by_key(|ranked| Reverse(ranked.score));
        ranked.truncate(top.into_inner());
        Matches(ranked)
    }
}

impl fmt::Display for Fit<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wanted: &MethodShape = self.role.as_ref();
        writeln!(formatter, "    {} {} fits {wanted}", AsRef::<Name>::as_ref(self.method), self.method.shape())
    }
}

impl fmt::Display for Ranked<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "score {}: {}", self.score, self.candidate)?;
        self.fits.iter().try_for_each(|fit| write!(formatter, "{fit}"))
    }
}

impl fmt::Display for Matches<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.first() {
            None => writeln!(formatter, "no existing traits, generic impls, ports or stores match"),
            Some(_) => self.0.iter().try_for_each(|ranked| write!(formatter, "{ranked}")),
        }
    }
}
