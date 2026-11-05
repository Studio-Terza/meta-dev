mod cover;
mod instantiation;

use std::collections::BTreeMap;

use super::{Accepted, Binders, Claim, Term};
use cover::Cover;
use instantiation::{Entailment, Instantiation};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    binders: Binders,
    subject: Term,
}

#[derive(Debug, Default)]
pub struct Claims {
    claims: Vec<Claim>,
}

impl<'claim> FromIterator<&'claim Claim> for Claims {
    fn from_iter<Found: IntoIterator<Item = &'claim Claim>>(found: Found) -> Claims {
        let merged = found.into_iter().fold(BTreeMap::<Key, Accepted>::new(), |mut merged, claim| {
            let key = Key { binders: claim.binders.clone(), subject: claim.subject.clone() };
            let accepted =
                merged.remove(&key).map_or_else(|| claim.accepted.clone(), |held| held.met(claim.accepted.clone()));
            merged.insert(key, accepted);
            merged
        });
        Claims {
            claims: merged
                .into_iter()
                .map(|(key, accepted)| Claim { binders: key.binders, subject: key.subject, accepted })
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
