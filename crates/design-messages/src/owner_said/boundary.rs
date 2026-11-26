use std::collections::BTreeSet;
use std::fmt;

use store_ports::{Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Body, Instant, SessionId, Uuid};

use super::{LastSaid, OwnerBefore, OwnerHistory, TurnEnds};
use crate::boundary::MessageRow;

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

impl Query for OwnerHistory {
    type Rows = Vec<Said>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Said>, StoreFailure> {
        MessageRow::owned(statements)
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
