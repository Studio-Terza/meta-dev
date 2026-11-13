use std::cmp::Reverse;
use std::fmt;

use code_model::MethodShape;
use newtype::Wrapping;

use crate::query::candidate::{Candidate, Offered};
use crate::query::term::Term;
use crate::query::top::Top;
use crate::role::Role;
use crate::score::Score;

#[derive(Clone, Debug)]
struct Fit<'facts> {
    role: &'facts Role,
    method: Offered<'facts>,
}

#[derive(Clone, Debug)]
struct Ranked<'facts> {
    score: Score,
    candidate: Candidate<'facts>,
    fits: Vec<Fit<'facts>>,
}

#[derive(Clone, Debug)]
pub struct Matches<'facts>(Vec<Ranked<'facts>>);

impl<'facts> Ranked<'facts> {
    fn scored(candidate: Candidate<'facts>, terms: &[Term], roles: &'facts [Role]) -> Ranked<'facts> {
        let shaped = roles.iter().map(|role| candidate.shaped(role)).fold(Score::default(), |sum, each| sum + each);
        let fits = roles
            .iter()
            .filter_map(|role| candidate.fitting(role).map(|method| Fit { role, method: method.clone() }))
            .collect();
        Ranked { score: candidate.named(terms) + shaped, candidate, fits }
    }
}

impl<'facts> Matches<'facts> {
    #[must_use]
    pub fn ranked(
        candidates: impl Iterator<Item = Candidate<'facts>>,
        terms: &[Term],
        roles: &'facts [Role],
        top: Top,
    ) -> Matches<'facts> {
        let mut ranked: Vec<Ranked<'facts>> = candidates
            .map(|candidate| Ranked::scored(candidate, terms, roles))
            .filter(|ranked| ranked.score > Score::default())
            .collect();
        ranked.sort_by_key(|entry| Reverse(entry.score));
        ranked.truncate(top.into_inner());
        Matches(ranked)
    }
}

impl fmt::Display for Fit<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wanted: &MethodShape = self.role.as_ref();
        writeln!(formatter, "    {} fits {wanted}", self.method)
    }
}

impl fmt::Display for Ranked<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "score {}: {}", self.score, self.candidate)?;
        self.fits.iter().try_for_each(|fit| write!(formatter, "{fit}"))
    }
}

impl fmt::Display for Matches<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.first() {
            None => writeln!(formatter, "no existing traits, generic impls, ports or stores match"),
            Some(_) => self.0.iter().try_for_each(|ranked| write!(formatter, "{ranked}")),
        }
    }
}
