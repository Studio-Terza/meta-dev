use design_model::{NodeId, Ordinal, Prefix, Segment};
use store_ports::{Columns, Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Uuid};

use super::{NextNode, Prefixed, Spoken, Started};
use crate::graph::NodeRow;

#[derive(Debug, Columns)]
struct SpokenRow {
    uuid: Uuid,
    author: Authorship,
    ordinal: Ordinal,
}

#[derive(Debug, Columns)]
struct StartedRow {
    uuid: Uuid,
}

impl From<SpokenRow> for Spoken {
    fn from(row: SpokenRow) -> Spoken {
        Spoken { segment: Segment::new(row.uuid, row.ordinal), author: row.author }
    }
}

impl Query for Prefixed {
    type Rows = Vec<Spoken>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Spoken>, StoreFailure> {
        let prefix = self.0;
        let rows = statements.rows(store_ports::query!(
            SpokenRow,
            r#"
SELECT
  messages.uuid,
  messages.author,
  segments.ordinal
FROM
  messages
  JOIN segments ON segments.message = messages.uuid
WHERE
  substr(messages.uuid, 1, length(?1)) = ?1
ORDER BY
  messages.uuid,
  segments.ordinal
"#,
            prefix: Prefix
        ))?;
        Ok(rows.into_iter().map(Spoken::from).collect())
    }
}

impl Query for Started {
    type Rows = Vec<Uuid>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Uuid>, StoreFailure> {
        let prefix = self.0;
        let rows = statements.rows(store_ports::query!(
            StartedRow,
            r#"
SELECT
  uuid
FROM
  messages
WHERE
  substr(uuid, 1, length(?1)) = ?1
ORDER BY
  uuid
"#,
            prefix: Prefix
        ))?;
        Ok(rows.into_iter().map(|row| row.uuid).collect())
    }
}

impl Query for NextNode {
    type Rows = NodeId;

    fn rows(self, statements: &mut Statements<'_>) -> Result<NodeId, StoreFailure> {
        NodeRow::next(statements)
    }
}
