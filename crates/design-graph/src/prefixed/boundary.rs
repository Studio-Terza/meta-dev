use std::collections::BTreeMap;

use design_model::{NodeId, Ordinal, Prefix, Segment};
use store_ports::{Columns, Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Uuid};

use super::{Cited, NextNode, Prefixed, Spoken, Started};
use crate::graph::NodeRow;
use design_closure::Listed;

#[derive(Debug, Columns)]
struct SpokenRow {
    uuid: Uuid,
    author: Authorship,
    ordinal: Ordinal,
}

#[derive(Debug, Columns)]
struct CitedRow {
    prefix: Prefix,
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

impl Query for Cited {
    type Rows = BTreeMap<Prefix, Vec<Spoken>>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<BTreeMap<Prefix, Vec<Spoken>>, StoreFailure> {
        let prefixes: Listed = self.0.iter().collect();
        let rows = statements.rows(store_ports::query!(
            CitedRow,
            r#"
WITH
sought (prefix) AS (
  SELECT
    CAST(json_each.value AS TEXT)
  FROM
    json_each(?1)
)
SELECT
  sought.prefix AS "prefix!: String",
  messages.uuid AS "uuid!: String",
  messages.author AS "author!: String",
  segments.ordinal AS "ordinal!: i64"
FROM
  sought
  JOIN messages ON substr(messages.uuid, 1, length(sought.prefix)) = sought.prefix
  JOIN segments ON segments.message = messages.uuid
ORDER BY
  sought.prefix,
  messages.uuid,
  segments.ordinal
"#,
            prefixes: Listed
        ))?;
        let empty = self.0.into_iter().map(|prefix| (prefix, Vec::new())).collect();
        Ok(rows.into_iter().fold(empty, |mut cited: BTreeMap<Prefix, Vec<Spoken>>, row| {
            cited
                .entry(row.prefix)
                .or_default()
                .push(Spoken { segment: Segment::new(row.uuid, row.ordinal), author: row.author });
            cited
        }))
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
