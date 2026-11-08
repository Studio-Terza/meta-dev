use std::collections::BTreeMap;

use design_model::{Ordinal, Segment};
use grammar::notes::{Date, Quote};
use store_ports::{Columns, Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Uuid};

use super::{Quoted, QuotedSegments};
use crate::Listed;

#[derive(Debug, Columns)]
struct QuotedRow {
    quote: Quote,
    message: Uuid,
    occurrence: Ordinal,
    ordinal: Ordinal,
    day: Date,
}

impl From<QuotedRow> for Quoted {
    fn from(row: QuotedRow) -> Quoted {
        Quoted { segment: Segment::new(row.message, row.ordinal), occurrence: row.occurrence, day: row.day }
    }
}

impl Query for QuotedSegments {
    type Rows = BTreeMap<Quote, Vec<Quoted>>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<BTreeMap<Quote, Vec<Quoted>>, StoreFailure> {
        let rows = QuotedRow::found(&self.0.iter().collect(), statements)?;
        let found = self.0.into_iter().map(|quote| (quote, Vec::new())).collect();
        Ok(rows.into_iter().fold(found, |mut found: BTreeMap<Quote, Vec<Quoted>>, row| {
            found.entry(row.quote.clone()).or_default().push(Quoted::from(row));
            found
        }))
    }
}

impl QuotedRow {
    fn found(quotes: &Listed, statements: &mut Statements<'_>) -> Result<Vec<QuotedRow>, StoreFailure> {
        let owner = Authorship::Owner;
        statements.rows(store_ports::query!(
            QuotedRow,
            r#"
WITH
RECURSIVE found (message, quote, occurrence, start) AS (
  SELECT
    messages.uuid,
    CAST(json_each.value AS TEXT),
    0,
    instr(messages.body, json_each.value)
  FROM
    messages
    JOIN json_each(?1) ON instr(messages.body, json_each.value) > 0
  WHERE
    messages.author = ?2
    AND length(json_each.value) > 0
  UNION ALL
  SELECT
    found.message,
    found.quote,
    found.occurrence + 1,
    found.start + instr(
      substr(messages.body, found.start + 1),
      found.quote
    )
  FROM
    found
    JOIN messages ON messages.uuid = found.message
  WHERE
    instr(
      substr(messages.body, found.start + 1),
      found.quote
    ) > 0
),
placed (message, ordinal, start, finish) AS (
  SELECT DISTINCT
    message,
    -1,
    1,
    1
  FROM
    found
  UNION ALL
  SELECT
    segments.message,
    segments.ordinal,
    CASE
      instr(
        substr(messages.body, placed.finish),
        segments.body
      )
      WHEN 0 THEN 0
      ELSE placed.finish + instr(
        substr(messages.body, placed.finish),
        segments.body
      ) - 1
    END,
    CASE
      instr(
        substr(messages.body, placed.finish),
        segments.body
      )
      WHEN 0 THEN placed.finish
      ELSE placed.finish + instr(
        substr(messages.body, placed.finish),
        segments.body
      ) - 1 + length(segments.body)
    END
  FROM
    placed
    JOIN messages ON messages.uuid = placed.message
    JOIN segments ON segments.message = placed.message
    AND segments.ordinal = placed.ordinal + 1
)
SELECT
  found.quote AS "quote!: String",
  found.message AS "message!: String",
  found.occurrence AS "occurrence!: i64",
  placed.ordinal AS "ordinal!: i64",
  substr(messages.at, 1, 10) AS "day!: String"
FROM
  found
  JOIN placed ON placed.message = found.message
  JOIN messages ON messages.uuid = found.message
WHERE
  placed.start > 0
  AND placed.finish > placed.start
  AND placed.start < found.start + length(found.quote)
  AND placed.finish > found.start
ORDER BY
  found.message,
  found.occurrence,
  placed.ordinal
"#,
            quotes: Listed,
            owner: Authorship
        ))
    }
}
