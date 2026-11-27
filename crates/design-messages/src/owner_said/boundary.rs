use std::fmt;

use grammar::Prose;
use invariants::Ignoring;
use ports::store::{Execute, Query, StoreFailure};
use ports::transcript::{Authorship, Body, Instant, Uuid};

use super::{LastSaid, OwnerHistory};
use crate::boundary::SaidRow;
use crate::boundary::ignored::Ignored;

#[derive(Clone, Debug, PartialEq)]
pub struct Said {
    uuid: Uuid,
    at: Instant,
    body: Body,
    asked: Option<Asked>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quote {
    uuid: Uuid,
    at: Instant,
    excerpt: Body,
    asked: Option<Asked>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Asked(Body);

#[derive(Clone, Copy, Debug)]
pub struct Unabridged<'said>(&'said Said);

impl Query for OwnerHistory {
    type Rows = Vec<Said>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Said>, StoreFailure> {
        SaidRow::owned(statements)
    }
}

impl<'said> From<&'said Said> for Unabridged<'said> {
    fn from(said: &'said Said) -> Unabridged<'said> {
        Unabridged(said)
    }
}

impl Query for LastSaid {
    type Rows = Option<Body>;

    fn rows(self, statements: &mut impl Execute) -> Result<Option<Body>, StoreFailure> {
        let rows = SaidRow::last_in(&self.0, Authorship::Lead, statements)?;
        Ok(rows.into_iter().next().map(|row| Said::from(row).body))
    }
}

impl Asked {
    const SHORT: usize = 8;

    fn answered(lead: Option<&Body>, reply: &Body) -> Option<Asked> {
        let replied: &str = reply.as_ref();
        lead.filter(|_said| Prose::read(replied).words() < Asked::SHORT)
            .and_then(|said| Prose::read(said.as_ref()).last_sentence().map(str::to_owned))
            .and_then(|sentence| Body::try_new(sentence).ignoring(Ignored::Unasked))
            .map(Asked)
    }
}

impl Said {
    #[must_use]
    pub fn quoted(&self, uuid: &Uuid, quote: &Body) -> Option<Quote> {
        let body: &str = self.body.as_ref();
        let excerpt: &str = quote.as_ref();
        (self.uuid == *uuid)
            .then_some(excerpt)
            .and_then(|text| text.chars().next())
            .filter(|_| body.contains(excerpt))
            .map(|_| Quote {
                uuid: self.uuid.clone(),
                at: self.at.clone(),
                excerpt: quote.clone(),
                asked: self.asked.clone(),
            })
    }

    pub(crate) fn new(uuid: Uuid, at: Instant, body: Body, lead: Option<&Body>) -> Said {
        let asked = Asked::answered(lead, &body);
        Said { uuid, at, body, asked }
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

impl fmt::Display for Quote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.asked {
            None => write!(formatter, "{} {}: \"{}\"", self.uuid, self.at, self.excerpt),
            Some(Asked(asked)) => write!(
                formatter,
                "{} {}: the lead said \"{}\" and the owner replied \"{}\"",
                self.uuid, self.at, asked, self.excerpt
            ),
        }
    }
}
