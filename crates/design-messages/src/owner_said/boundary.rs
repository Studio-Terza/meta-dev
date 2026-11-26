use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::{fmt, slice};

use store_ports::{Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Body, Instant, SessionId, Uuid};

use super::{LastSaid, OwnerBefore, OwnerSaid, TurnEnds};
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

#[derive(Clone, Debug, PartialEq)]
pub struct TurnEnd {
    asked: Said,
    reply: Said,
}

#[derive(Clone, Copy, Debug)]
pub struct Unabridged<'said>(&'said Said);

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
            .filter(|said| self.before.as_ref().is_none_or(|bound| said.at < *bound))
            .filter(|said| seen.insert(said.uuid.clone()))
            .take(OwnerSaid::MOST)
            .collect())
    }
}

impl Query for OwnerBefore {
    type Rows = Vec<Said>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Said>, StoreFailure> {
        Ok(MessageRow::owner_before(&self.0, statements)?.into_iter().map(Said::from).collect())
    }
}

impl Query for TurnEnds {
    type Rows = Vec<TurnEnd>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<TurnEnd>, StoreFailure> {
        let replies = MessageRow::owner_between(&self.session, &self.from, &self.until, statements)?;
        let ends = replies
            .into_iter()
            .map(Said::from)
            .map(|reply| TurnEnd::read(&self.session, reply, statements))
            .collect::<Result<Vec<Option<TurnEnd>>, StoreFailure>>()?;
        let mut seen = BTreeSet::new();
        Ok(ends.into_iter().flatten().filter(|end| seen.insert(end.asked.uuid.clone())).collect())
    }
}

impl TurnEnd {
    fn read(
        session: &SessionId,
        reply: Said,
        statements: &mut Statements<'_>,
    ) -> Result<Option<TurnEnd>, StoreFailure> {
        let asked = MessageRow::lead_before(session, &reply.at, statements)?;
        Ok(asked.into_iter().next().map(|row| TurnEnd { asked: Said::from(row), reply }))
    }

    #[must_use]
    pub fn asked(&self) -> &Said {
        &self.asked
    }

    #[must_use]
    pub fn replied(&self) -> Unabridged<'_> {
        Unabridged(&self.reply)
    }
}

impl<'said> From<&'said Said> for Unabridged<'said> {
    fn from(said: &'said Said) -> Unabridged<'said> {
        Unabridged(said)
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
        [self.terms.first()]
            .into_iter()
            .chain(self.terms.rest())
            .map(|term| {
                Hit::matching(term, scope, statements).map(|hits| Hit::best_per_message(hits, self.before.as_ref()))
            })
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

impl AsRef<Instant> for Said {
    fn as_ref(&self) -> &Instant {
        &self.at
    }
}

impl AsRef<Body> for Said {
    fn as_ref(&self) -> &Body {
        &self.body
    }
}

impl AsRef<Uuid> for Quote {
    fn as_ref(&self) -> &Uuid {
        &self.uuid
    }
}

impl fmt::Display for Unabridged<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{} {}] {}", self.0.uuid, self.0.at, self.0.body)
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
