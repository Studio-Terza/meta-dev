use design_model::{NodeId, Ordinal, Provenance, SpanRelation};
use store_ports::{Columns, Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Body, Uuid};

use super::{CurrentSpans, Uttered};

#[derive(Debug, Columns)]
struct UtteredRow {
    node: NodeId,
    relation: SpanRelation,
    message: Uuid,
    ordinal: Ordinal,
    author: Authorship,
    provenance: Provenance,
    body: Body,
}

impl From<UtteredRow> for Uttered {
    fn from(row: UtteredRow) -> Uttered {
        let UtteredRow { node, relation, message, ordinal, author, provenance, body } = row;
        Uttered { node, relation, message, ordinal, author, provenance, body }
    }
}

impl Query for CurrentSpans {
    type Rows = Vec<Uttered>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Uttered>, StoreFailure> {
        let relation = self.0;
        let rows = statements.rows(store_ports::query!(
            UtteredRow,
            r#"
SELECT
  current_spans.node AS "node!: i64",
  current_spans.relation AS "relation!: String",
  current_spans.message AS "message!: String",
  current_spans.ordinal AS "ordinal!: i64",
  messages.author,
  current_spans.provenance AS "provenance!: String",
  segments.body
FROM
  current_spans
  JOIN segments ON segments.message = current_spans.message
  AND segments.ordinal = current_spans.ordinal
  JOIN messages ON messages.uuid = current_spans.message
WHERE
  current_spans.relation = ?1
ORDER BY
  current_spans.node,
  messages.at,
  current_spans.message,
  current_spans.ordinal
"#,
            relation: SpanRelation
        ))?;
        Ok(rows.into_iter().map(Uttered::from).collect())
    }
}
