use design_model::{
    Confidence, Explicitness, NodeId, Ordinal, Provenance, Sense, SenseKind, SpanRelation, Specificity,
};
use store_ports::{Columns, Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Body, Uuid};

use super::{CurrentSpans, Placement};

#[derive(Debug, Columns)]
struct PlacementRow {
    node: NodeId,
    relation: SpanRelation,
    message: Uuid,
    ordinal: Ordinal,
    author: Authorship,
    body: Body,
    provenance: Provenance,
    kind: Option<SenseKind>,
    specificity: Option<Specificity>,
    confidence: Option<Confidence>,
    explicitness: Option<Explicitness>,
}

impl From<PlacementRow> for Placement {
    fn from(row: PlacementRow) -> Placement {
        let sense = row.kind.zip(row.specificity).zip(row.confidence).zip(row.explicitness).map(
            |(((kind, specificity), confidence), explicitness)| Sense::new(kind, specificity, confidence, explicitness),
        );
        Placement {
            node: row.node,
            relation: row.relation,
            message: row.message,
            ordinal: row.ordinal,
            author: row.author,
            body: row.body,
            sense,
            provenance: row.provenance,
        }
    }
}

impl Placement {
    pub(crate) fn placed_on(node: NodeId, statements: &mut Statements<'_>) -> Result<Vec<Placement>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            PlacementRow,
            r#"
SELECT
  span_edges.node,
  span_edges.relation,
  span_edges.message,
  span_edges.ordinal,
  messages.author,
  segments.body,
  span_edges.provenance,
  senses.kind,
  senses.specificity,
  senses.confidence,
  senses.explicitness
FROM
  span_edges
  JOIN segments ON segments.message = span_edges.message
  AND segments.ordinal = span_edges.ordinal
  JOIN messages ON messages.uuid = span_edges.message
  LEFT JOIN senses ON senses.message = span_edges.message
  AND senses.ordinal = span_edges.ordinal
  AND NOT EXISTS (
    SELECT
      1
    FROM
      senses AS later
    WHERE
      later.supersedes = senses.id
  )
WHERE
  span_edges.node = ?1
ORDER BY
  span_edges.rowid
"#,
            node: NodeId
        ))?;
        Ok(rows.into_iter().map(Placement::from).collect())
    }
}

impl Query for CurrentSpans {
    type Rows = Vec<Placement>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Placement>, StoreFailure> {
        let relation = self.0;
        let rows = statements.rows(store_ports::query!(
            PlacementRow,
            r#"
SELECT
  current_spans.node AS "node!: i64",
  current_spans.relation AS "relation!: String",
  current_spans.message AS "message!: String",
  current_spans.ordinal AS "ordinal!: i64",
  messages.author,
  segments.body,
  current_spans.provenance AS "provenance!: String",
  senses.kind AS "kind?: String",
  senses.specificity AS "specificity?: String",
  senses.confidence AS "confidence?: String",
  senses.explicitness AS "explicitness?: String"
FROM
  current_spans
  JOIN segments ON segments.message = current_spans.message
  AND segments.ordinal = current_spans.ordinal
  JOIN messages ON messages.uuid = current_spans.message
  LEFT JOIN senses ON senses.message = current_spans.message
  AND senses.ordinal = current_spans.ordinal
  AND NOT EXISTS (
    SELECT
      1
    FROM
      senses AS later
    WHERE
      later.supersedes = senses.id
  )
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
        Ok(rows.into_iter().map(Placement::from).collect())
    }
}
