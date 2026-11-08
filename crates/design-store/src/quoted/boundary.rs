use std::collections::BTreeMap;

use design_model::{Ordinal, Segment};
use grammar::notes::{Date, Quote};
use store_ports::{Columns, Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Uuid};

use super::{Quoted, QuotedSegments};

#[derive(Debug, Columns)]
struct QuotedRow {
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
        self.0.into_iter().map(|quote| QuotedRow::found(&quote, statements).map(|found| (quote, found))).collect()
    }
}

impl QuotedRow {
    fn found(quote: &Quote, statements: &mut Statements<'_>) -> Result<Vec<Quoted>, StoreFailure> {
        let quote = quote.clone();
        let owner = Authorship::Owner;
        let rows = statements.rows(store_ports::query!(
            QuotedRow,
            r#"
WITH
RECURSIVE spoken (uuid, body, day) AS (
  SELECT
    uuid,
    body,
    substr(at, 1, 10)
  FROM
    messages
  WHERE
    author = ?2
    AND length(?1) > 0
    AND instr(body, ?1) > 0
),
found (message, occurrence, start) AS (
  SELECT
    uuid,
    0,
    instr(body, ?1)
  FROM
    spoken
  UNION ALL
  SELECT
    found.message,
    found.occurrence + 1,
    found.start + instr(substr(spoken.body, found.start + 1), ?1)
  FROM
    found
    JOIN spoken ON spoken.uuid = found.message
  WHERE
    instr(substr(spoken.body, found.start + 1), ?1) > 0
),
placed (message, ordinal, start, finish) AS (
  SELECT
    uuid,
    -1,
    1,
    1
  FROM
    spoken
  UNION ALL
  SELECT
    segments.message,
    segments.ordinal,
    CASE
      instr(
        substr(spoken.body, placed.finish),
        segments.body
      )
      WHEN 0 THEN 0
      ELSE placed.finish + instr(
        substr(spoken.body, placed.finish),
        segments.body
      ) - 1
    END,
    CASE
      instr(
        substr(spoken.body, placed.finish),
        segments.body
      )
      WHEN 0 THEN placed.finish
      ELSE placed.finish + instr(
        substr(spoken.body, placed.finish),
        segments.body
      ) - 1 + length(segments.body)
    END
  FROM
    placed
    JOIN spoken ON spoken.uuid = placed.message
    JOIN segments ON segments.message = placed.message
    AND segments.ordinal = placed.ordinal + 1
)
SELECT
  found.message AS "message!: String",
  found.occurrence AS "occurrence!: i64",
  placed.ordinal AS "ordinal!: i64",
  spoken.day AS "day!: String"
FROM
  found
  JOIN placed ON placed.message = found.message
  JOIN spoken ON spoken.uuid = found.message
WHERE
  placed.start > 0
  AND placed.finish > placed.start
  AND placed.start < found.start + length(?1)
  AND placed.finish > found.start
ORDER BY
  found.message,
  found.occurrence,
  placed.ordinal
"#,
            quote: Quote,
            owner: Authorship
        ))?;
        Ok(rows.into_iter().map(Quoted::from).collect())
    }
}
