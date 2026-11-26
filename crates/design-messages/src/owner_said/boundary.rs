use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::{fmt, slice};

use store_ports::{Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Body, Instant, Uuid};

use super::{LastSaid, OwnerSaid};
use crate::Scope;
use crate::boundary::{Hit, MessageRow};

#[derive(Clone, Debug, PartialEq)]
pub struct Said {
    uuid: Uuid,
    at: Instant,
    body: Body,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quote {
    uuid: Uuid,
    at: Instant,
    excerpt: Body,
}

#[derive(Clone, Debug)]
struct Matched {
    uuid: Uuid,
    terms: usize,
    rank: f64,
}

impl Query for OwnerSaid {
    type Rows = Vec<Said>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Said>, StoreFailure> {
        let owned = self.best(Scope::Owner, statements)?;
        let everyone = self.best(Scope::Everyone, statements)?;
        let named = owned
            .iter()
            .flat_map(|best| Matched::ranked(slice::from_ref(best)).into_iter().take(OwnerSaid::SHARPEST))
            .chain(Matched::ranked(&owned).into_iter().take(OwnerSaid::EACH))
            .chain(Matched::ranked(&everyone).into_iter().take(OwnerSaid::EACH))
            .map(|uuid| MessageRow::named(&uuid, statements))
            .collect::<Result<Vec<Vec<MessageRow>>, StoreFailure>>()?;
        let answering = named
            .into_iter()
            .flatten()
            .map(|row| row.answering(statements))
            .collect::<Result<Vec<Vec<MessageRow>>, StoreFailure>>()?;
        let mut seen = BTreeSet::new();
        Ok(answering
            .into_iter()
            .flatten()
            .map(Said::from)
            .filter(|said| seen.insert(said.uuid.clone()))
            .take(OwnerSaid::MOST)
            .collect())
    }
}

impl Query for LastSaid {
    type Rows = Option<Body>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Option<Body>, StoreFailure> {
        let rows = MessageRow::last_in(&self.0, Authorship::Lead, statements)?;
        Ok(rows.into_iter().next().map(|row| Said::from(row).body))
    }
}

impl OwnerSaid {
    fn best(&self, scope: Scope, statements: &mut Statements<'_>) -> Result<Vec<BTreeMap<Uuid, f64>>, StoreFailure> {
        [self.0.first()]
            .into_iter()
            .chain(self.0.rest())
            .map(|term| Hit::matching(term, scope, statements).map(Hit::best_per_message))
            .collect()
    }
}

impl Matched {
    fn ranked(best: &[BTreeMap<Uuid, f64>]) -> Vec<Uuid> {
        let mut each: Vec<Matched> = best.iter().flat_map(Matched::each).collect();
        each.sort_by(|left, right| left.uuid.cmp(&right.uuid));
        let mut ranked: Vec<Matched> =
            each.chunk_by(|left, right| left.uuid == right.uuid).filter_map(Matched::summed).collect();
        ranked.sort_by(Matched::ordered);
        ranked.into_iter().map(|matched| matched.uuid).collect()
    }

    fn each(best: &BTreeMap<Uuid, f64>) -> Vec<Matched> {
        best.iter().map(|(uuid, rank)| Matched { uuid: uuid.clone(), terms: 1, rank: *rank }).collect()
    }

    fn summed(chunk: &[Matched]) -> Option<Matched> {
        chunk.first().map(|first| Matched {
            uuid: first.uuid.clone(),
            terms: chunk.len(),
            rank: chunk.iter().map(|matched| matched.rank).sum(),
        })
    }

    fn ordered(&self, other: &Matched) -> Ordering {
        self.rank
            .total_cmp(&other.rank)
            .then_with(|| other.terms.cmp(&self.terms))
            .then_with(|| self.uuid.cmp(&other.uuid))
    }
}

impl Said {
    const SHOWN: usize = 600;

    #[must_use]
    pub fn quoted(&self, uuid: &Uuid, quote: &Body) -> Option<Quote> {
        let body: &str = self.body.as_ref();
        let excerpt: &str = quote.as_ref();
        (self.uuid == *uuid)
            .then_some(excerpt)
            .and_then(|text| text.chars().next())
            .filter(|_| body.contains(excerpt))
            .map(|_| Quote { uuid: self.uuid.clone(), at: self.at.clone(), excerpt: quote.clone() })
    }

    pub(crate) fn new(uuid: Uuid, at: Instant, body: Body) -> Said {
        Said { uuid, at, body }
    }
}

impl AsRef<Uuid> for Said {
    fn as_ref(&self) -> &Uuid {
        &self.uuid
    }
}

impl fmt::Display for Said {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text: &str = self.body.as_ref();
        let shown: String = text.chars().take(Said::SHOWN).collect();
        write!(formatter, "[{} {}] {shown}", self.uuid, self.at)
    }
}

impl fmt::Display for Quote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}: \"{}\"", self.uuid, self.at, self.excerpt)
    }
}
