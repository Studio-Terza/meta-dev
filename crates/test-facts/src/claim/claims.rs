mod cover;
mod instantiation;

use std::collections::BTreeMap;

use super::{Accepted, Binders, Claim, Guard, Term};
use cover::Cover;
use instantiation::{Entailment, Instantiation};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    binders: Binders,
    guard: Guard,
    subject: Term,
}

#[derive(Debug, Default)]
struct Pool(Vec<Accepted>);

impl Pool {
    fn including(self, next: &Accepted) -> Pool {
        let chosen = self.0.iter().find(|held| (*held).clone().met(next.clone()).consistent().is_some()).cloned();
        let merged = chosen.as_ref().map_or_else(|| next.clone(), |held| held.clone().met(next.clone()));
        Pool(self.0.into_iter().filter(|held| Some(held) != chosen.as_ref()).chain([merged]).collect())
    }
}

#[derive(Debug, Default)]
pub struct Claims {
    claims: Vec<Claim>,
}

impl<'claim> FromIterator<&'claim Claim> for Claims {
    fn from_iter<Found: IntoIterator<Item = &'claim Claim>>(found: Found) -> Claims {
        let merged = found.into_iter().fold(BTreeMap::<Key, Pool>::new(), |mut merged, claim| {
            let key =
                Key { binders: claim.binders.clone(), guard: claim.guard.clone(), subject: claim.subject.clone() };
            let pool = merged.remove(&key).unwrap_or_default().including(&claim.accepted);
            merged.insert(key, pool);
            merged
        });
        Claims {
            claims: merged
                .into_iter()
                .flat_map(|(key, pool)| {
                    pool.0.into_iter().map(move |accepted| Claim {
                        binders: key.binders.clone(),
                        guard: key.guard.clone(),
                        subject: key.subject.clone(),
                        accepted,
                    })
                })
                .collect(),
        }
    }
}

impl Claims {
    #[must_use]
    pub fn lost<'claim>(&self, before: &'claim [Claim]) -> Vec<&'claim Claim> {
        before.iter().filter(|claim| self.entailing(claim).is_none()).collect()
    }

    fn entailing(&self, claim: &Claim) -> Option<Entailment> {
        self.claims
            .iter()
            .find_map(|after| Instantiation::matching(after, claim).entailment())
            .or_else(|| Cover::ranged(claim).iter().find_map(|cover| cover.covered_by(&self.claims)))
    }
}
