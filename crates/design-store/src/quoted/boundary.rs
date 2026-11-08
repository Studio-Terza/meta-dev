use std::collections::BTreeMap;

use design_model::{Ordinal, Segment};
use grammar::notes::{Date, Quote};
use store_ports::{Columns, Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Uuid};

use super::{Quoted, QuotedSegments};

#[derive(Debug, Columns)]
struct QuotedRow {
    message: Uuid,
    ordinal: Ordinal,
    day: Date,
}

impl From<QuotedRow> for Quoted {
    fn from(row: QuotedRow) -> Quoted {
        Quoted { segment: Segment::new(row.message, row.ordinal), day: row.day }
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
SELECT
  segments.message,
  segments.ordinal,
  substr(messages.at, 1, 10) AS "day!: String"
FROM
  messages
  JOIN segments ON segments.message = messages.uuid
WHERE
  messages.author = ?2
  AND length(?1) > 0
  AND length(segments.body) > 0
  AND instr(messages.body, ?1) > 0
  AND instr(messages.body, segments.body) < instr(messages.body, ?1) + length(?1)
  AND instr(messages.body, segments.body) + length(segments.body) > instr(messages.body, ?1)
ORDER BY
  segments.message,
  segments.ordinal
"#,
            quote: Quote,
            owner: Authorship
        ))?;
        Ok(rows.into_iter().map(Quoted::from).collect())
    }
}
