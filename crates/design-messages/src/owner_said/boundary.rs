use std::collections::BTreeMap;
use std::fmt;

use grammar::prose::Prose;
use invariants::Ignoring;
use port_core::Op;
use store_ports::{Columns, Execute, Query, StoreFailure};
use transcript_ports::{Authorship, Body, Instant, SessionId, Uuid};

use crate::boundary::ignored::Ignored;

#[derive(Clone, Copy, Debug, Op)]
pub struct OwnerHistory;

#[derive(Clone, Debug, Op)]
pub struct LastSaid(SessionId);

#[derive(Debug, Columns)]
struct SaidRow {
    uuid: Uuid,
    at: Instant,
    author: Authorship,
    body: Body,
}

#[derive(Debug, Default)]
struct Exchanges {
    lead: Option<Body>,
    owned: Vec<Said>,
}

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

impl From<SessionId> for LastSaid {
    fn from(session: SessionId) -> LastSaid {
        LastSaid(session)
    }
}

impl From<SaidRow> for Said {
    fn from(row: SaidRow) -> Said {
        Said::new(row.uuid, row.at, row.body, None)
    }
}

impl Exchanges {
    fn heard(self, row: SaidRow) -> Exchanges {
        match row.author {
            Authorship::Lead => Exchanges { lead: Some(row.body), ..self },
            Authorship::Owner => {
                let said = Said::new(row.uuid, row.at, row.body, self.lead.as_ref());
                let mut owned = self.owned;
                owned.push(said);
                Exchanges { owned, ..self }
            }
        }
    }
}

impl SaidRow {
    fn last_in(
        session: &SessionId,
        author: Authorship,
        statements: &mut impl Execute,
    ) -> Result<Vec<SaidRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            SaidRow,
            r#"
SELECT
  uuid,
  at,
  author,
  body
FROM
  messages
WHERE
  session = ?1
  AND author = ?2
ORDER BY
  at DESC
LIMIT
  1
"#,
            session: SessionId,
            author: Authorship
        ))
    }

    fn owned(statements: &mut impl Execute) -> Result<Vec<Said>, StoreFailure> {
        let ordered: BTreeMap<(Instant, Uuid), SaidRow> =
            SaidRow::all(statements)?.into_iter().map(|row| ((row.at.clone(), row.uuid.clone()), row)).collect();
        Ok(ordered.into_values().fold(Exchanges::default(), Exchanges::heard).owned)
    }

    fn all(statements: &mut impl Execute) -> Result<Vec<SaidRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            SaidRow,
            r#"
SELECT
  uuid,
  at,
  author,
  body
FROM
  messages
"#
        ))
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
        lead.filter(|_said| Prose::from(replied).words() < Asked::SHORT)
            .and_then(|said| Prose::from(said.as_ref()).answered().map(str::to_owned))
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
        write!(formatter, "[{} {}] ", self.0.uuid, self.0.at)?;
        self.0.asked.iter().try_for_each(|Asked(asked)| write!(formatter, "(replying to the lead's \"{asked}\") "))?;
        write!(formatter, "{}", self.0.body)
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
