use std::fmt;

use design_model::Ordinal;
use newtype::{Arbitrary, Display, Newtype};
use store_ports::{Batch, Columns, Execute, Prepared, Query, StoreFailure, Written};
use transcript_ports::{Body, Instant, Uuid};

use super::{Idealised, Ideals};
use crate::Quote;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdealId(i64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ideal {
    id: IdealId,
    superseded: Option<IdealId>,
    quotes: Vec<Quote>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Escaped<'text>(&'text str);

#[derive(Debug, Columns)]
struct IdealQuoteRow {
    ideal: IdealId,
    superseded_by: Option<IdealId>,
    message: Uuid,
    at: Instant,
    excerpt: Body,
}

impl Ideal {
    #[must_use]
    pub fn new(id: IdealId, quotes: Vec<Quote>) -> Ideal {
        Ideal { id, superseded: None, quotes }
    }

    #[must_use]
    pub fn superseded_by(self, later: IdealId) -> Ideal {
        Ideal { superseded: Some(later), ..self }
    }

    #[must_use]
    pub fn superseded(&self) -> Option<IdealId> {
        self.superseded
    }

    #[must_use]
    pub fn quotes(&self) -> &[Quote] {
        &self.quotes
    }

    fn inserted(&self) -> Vec<Prepared<()>> {
        let Ideal { id, superseded, quotes } = self;
        let listed = store_ports::query!(
            r#"
INSERT INTO
  ideals (id, superseded_by)
VALUES
  (?1, ?2)
"#,
            id: IdealId,
            superseded: Option<IdealId>
        );
        let quoted = quotes.iter().zip(0_u32..).map(|(quote, number)| {
            let place = Ordinal::new(number);
            let message: &Uuid = quote.as_ref();
            let excerpt: &Body = quote.as_ref();
            store_ports::query!(
                r#"
INSERT INTO
  ideal_quotes (ideal, place, message, excerpt)
VALUES
  (?1, ?2, ?3, ?4)
"#,
                id: IdealId,
                place: Ordinal,
                message: Uuid,
                excerpt: Body
            )
        });
        [listed].into_iter().chain(quoted).collect()
    }
}

impl AsRef<IdealId> for Ideal {
    fn as_ref(&self) -> &IdealId {
        &self.id
    }
}

impl IdealQuoteRow {
    fn quote(&self) -> Quote {
        Quote::new(self.message.clone(), self.at.clone(), self.excerpt.clone())
    }
}

impl Ideal {
    fn grouped(group: &[IdealQuoteRow]) -> Option<Ideal> {
        group.first().map(|first| Ideal {
            id: first.ideal,
            superseded: first.superseded_by,
            quotes: group.iter().map(IdealQuoteRow::quote).collect(),
        })
    }
}

impl<'text> From<&'text str> for Escaped<'text> {
    fn from(text: &'text str) -> Escaped<'text> {
        Escaped(text)
    }
}

impl Batch for Idealised {
    type Failure = StoreFailure;

    fn written(self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        let Idealised(ideals) = self;
        let cleared = [
            store_ports::query!(
                r#"
DELETE FROM
  ideal_quotes
"#
            ),
            store_ports::query!(
                r#"
DELETE FROM
  ideals
"#
            ),
        ];
        statements.written_all(cleared.into_iter().chain(ideals.iter().flat_map(Ideal::inserted)))
    }
}

impl Query for Ideals {
    type Rows = Vec<Ideal>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Ideal>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            IdealQuoteRow,
            r#"
SELECT
  ideals.id AS ideal,
  ideals.superseded_by,
  ideal_quotes.message,
  messages.at,
  ideal_quotes.excerpt
FROM
  ideals
  JOIN ideal_quotes ON ideal_quotes.ideal = ideals.id
  JOIN messages ON messages.uuid = ideal_quotes.message
ORDER BY
  ideals.id,
  ideal_quotes.place
"#,
        ))?;
        Ok(rows.chunk_by(|left, right| left.ideal == right.ideal).filter_map(Ideal::grouped).collect())
    }
}

impl fmt::Display for Escaped<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&serde_json::to_string(self.0).map_err(|_unwritten| fmt::Error)?)
    }
}

impl fmt::Display for Ideal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let standing = self.superseded.map_or_else(|| String::from("current"), |later| format!("superseded:i{later}"));
        let lines: Vec<String> = self
            .quotes
            .iter()
            .map(|quote| {
                let uuid: &Uuid = quote.as_ref();
                let excerpt: &Body = quote.as_ref();
                format!("i{}\t{standing}\t{uuid}\t{}", self.id, Escaped::from(excerpt.as_ref()))
            })
            .collect();
        write!(formatter, "{}", lines.join("\n"))
    }
}
