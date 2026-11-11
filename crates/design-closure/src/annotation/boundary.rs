use design_model::{
    Confidence, Explicitness, NodeId, Ordinal, Provenance, Segment, SegmentRelation, Sense, SenseKind, SpanRelation,
    Specificity, Standing,
};
use store_ports::{Columns as Row, Statements, StoreFailure};
use transcript_ports::Uuid;

use super::{Annotation, Remark};

#[derive(Debug, Row)]
pub(crate) struct PlacedRow {
    message: Uuid,
    ordinal: Ordinal,
    relation: SpanRelation,
    node: NodeId,
    provenance: Provenance,
}

#[derive(Debug, Row)]
pub(crate) struct SensedRow {
    message: Uuid,
    ordinal: Ordinal,
    kind: SenseKind,
    specificity: Specificity,
    confidence: Confidence,
    explicitness: Explicitness,
    provenance: Provenance,
    standing: Standing,
}

#[derive(Debug, Row)]
pub(crate) struct RelatedRow {
    source_message: Uuid,
    source_ordinal: Ordinal,
    relation: SegmentRelation,
    target_message: Uuid,
    target_ordinal: Ordinal,
    provenance: Provenance,
}

#[derive(Debug, Row)]
pub(crate) struct ConversationalRow {
    message: Uuid,
    ordinal: Ordinal,
    provenance: Provenance,
}

impl From<PlacedRow> for Annotation {
    fn from(row: PlacedRow) -> Annotation {
        let segment = Segment::new(row.message, row.ordinal);
        let remark = Remark::Placed { segment, node: row.node, relation: row.relation };
        Annotation::new(remark, row.provenance, Standing::Current)
    }
}

impl From<SensedRow> for Annotation {
    fn from(row: SensedRow) -> Annotation {
        let segment = Segment::new(row.message, row.ordinal);
        let sense = Sense::new(row.kind, row.specificity, row.confidence, row.explicitness);
        Annotation::new(Remark::Sensed { segment, sense }, row.provenance, row.standing)
    }
}

impl From<RelatedRow> for Annotation {
    fn from(row: RelatedRow) -> Annotation {
        let source = Segment::new(row.source_message, row.source_ordinal);
        let target = Segment::new(row.target_message, row.target_ordinal);
        let remark = Remark::Related { source, target, relation: row.relation };
        Annotation::new(remark, row.provenance, Standing::Current)
    }
}

impl From<ConversationalRow> for Annotation {
    fn from(row: ConversationalRow) -> Annotation {
        let segment = Segment::new(row.message, row.ordinal);
        Annotation::new(Remark::Conversational { segment }, row.provenance, Standing::Current)
    }
}

impl PlacedRow {
    fn stored_for(message: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<PlacedRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            PlacedRow,
            r#"
SELECT
  message,
  ordinal,
  relation,
  node,
  provenance
FROM
  span_edges
WHERE
  message = ?1
ORDER BY
  ordinal,
  relation,
  node
"#,
            message: Uuid
        ))
    }
}

impl SensedRow {
    fn stored_for(message: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<SensedRow>, StoreFailure> {
        let current = Standing::Current;
        let superseded = Standing::Superseded;
        statements.rows(store_ports::query!(
            SensedRow,
            r#"
SELECT
  message,
  ordinal,
  kind,
  specificity,
  confidence,
  explicitness,
  provenance,
  CASE
    WHEN EXISTS (
      SELECT
        1
      FROM
        senses AS later
      WHERE
        later.supersedes = senses.id
    ) THEN ?2
    ELSE ?3
  END AS "standing!: String"
FROM
  senses
WHERE
  message = ?1
ORDER BY
  ordinal,
  id
"#,
            message: Uuid,
            superseded: Standing,
            current: Standing
        ))
    }
}

impl RelatedRow {
    fn stored_for(message: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<RelatedRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            RelatedRow,
            r#"
SELECT
  source_message,
  source_ordinal,
  relation,
  target_message,
  target_ordinal,
  provenance
FROM
  segment_links
WHERE
  source_message = ?1
ORDER BY
  source_ordinal,
  relation,
  target_message,
  target_ordinal
"#,
            message: Uuid
        ))
    }
}

impl ConversationalRow {
    fn stored_for(message: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<ConversationalRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            ConversationalRow,
            r#"
SELECT
  message,
  ordinal,
  provenance
FROM
  conversational
WHERE
  message = ?1
ORDER BY
  ordinal
"#,
            message: Uuid
        ))
    }
}

impl Annotation {
    pub fn stored_for(message: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<Annotation>, StoreFailure> {
        let placed = PlacedRow::stored_for(message, statements)?;
        let sensed = SensedRow::stored_for(message, statements)?;
        let related = RelatedRow::stored_for(message, statements)?;
        let conversational = ConversationalRow::stored_for(message, statements)?;
        let annotations = placed
            .into_iter()
            .map(Annotation::from)
            .chain(sensed.into_iter().map(Annotation::from))
            .chain(related.into_iter().map(Annotation::from))
            .chain(conversational.into_iter().map(Annotation::from));
        Ok(annotations.collect())
    }
}
