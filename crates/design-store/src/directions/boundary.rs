use design_model::{
    Confidence, Entered, Explicitness, Node, NodeId, NodeRelation, Ordinal, Provenance, Segment, SegmentRelation,
    Sense, SenseKind, SpanRelation, Specificity, Standing, Supersession, Vague,
};
use newtype::Counter;
use store_ports::{Batch, Columns as Row, Statements, StoreFailure, Written};
use transcript_ports::Uuid;

use super::Directions;
use crate::acyclic::{Closing, Keyed, Refusal, SegmentCorrects, SegmentSupersedes};
use crate::annotation::Remark;
use crate::graph::{NodeRow, Resolved};
use crate::{Annotation, Linked};

#[derive(Debug)]
pub(super) struct Annotating<'held, 'run> {
    provenance: Entered,
    statements: &'held mut Statements<'run>,
}

#[derive(Clone, Copy, Debug)]
struct Linking<'ends> {
    source: &'ends Segment,
    target: &'ends Segment,
    relation: SegmentRelation,
    provenance: Entered,
}

#[derive(Debug)]
struct LinkColumns {
    source: Columns,
    relation: SegmentRelation,
    target: Columns,
    provenance: Entered,
}

#[derive(Debug, Row)]
pub(crate) struct Columns {
    message: Uuid,
    ordinal: Ordinal,
}

#[derive(Debug, Row)]
struct KeyRow {
    key: Keyed,
}

#[derive(Debug, Row)]
pub(crate) struct PlacedRow {
    message: Uuid,
    ordinal: Ordinal,
    relation: SpanRelation,
    node: NodeId,
    provenance: Entered,
}

#[derive(Debug, Row)]
pub(crate) struct SensedRow {
    message: Uuid,
    ordinal: Ordinal,
    kind: SenseKind,
    specificity: Specificity,
    confidence: Confidence,
    explicitness: Explicitness,
    provenance: Entered,
    standing: Standing,
}

#[derive(Debug, Row)]
pub(crate) struct RelatedRow {
    source_message: Uuid,
    source_ordinal: Ordinal,
    relation: SegmentRelation,
    target_message: Uuid,
    target_ordinal: Ordinal,
    provenance: Entered,
}

#[derive(Debug, Row)]
pub(crate) struct ConversationalRow {
    message: Uuid,
    ordinal: Ordinal,
    provenance: Entered,
}

impl Columns {
    pub(crate) fn rowid(self, statements: &mut Statements<'_>) -> Result<Keyed, StoreFailure> {
        let Columns { message, ordinal } = self;
        let row = statements.one(store_ports::query!(
            KeyRow,
            r#"
SELECT
  rowid AS "key!: i64"
FROM
  segments
WHERE
  message = ?1
  AND ordinal = ?2
"#,
            message: Uuid,
            ordinal: Ordinal
        ))?;
        Ok(row.key)
    }

    pub(crate) fn covered(message: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<Segment>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            Columns,
            r#"
SELECT
  message AS "message!: String",
  ordinal AS "ordinal!: i64"
FROM
  (
    SELECT
      message,
      ordinal
    FROM
      span_edges
    WHERE
      message = ?1
    UNION
    SELECT
      message,
      ordinal
    FROM
      senses
    WHERE
      message = ?1
    UNION
    SELECT
      message,
      ordinal
    FROM
      conversational
    WHERE
      message = ?1
    UNION
    SELECT
      source_message,
      source_ordinal
    FROM
      segment_links
    WHERE
      source_message = ?1
    UNION
    SELECT
      target_message,
      target_ordinal
    FROM
      segment_links
    WHERE
      target_message = ?1
  )
ORDER BY
  ordinal
"#,
            message: Uuid
        ))?;
        Ok(rows.into_iter().map(Segment::from).collect())
    }
}

impl From<Columns> for Segment {
    fn from(columns: Columns) -> Segment {
        Segment::new(columns.message, columns.ordinal)
    }
}

impl From<PlacedRow> for Annotation {
    fn from(row: PlacedRow) -> Annotation {
        let segment = Segment::new(row.message, row.ordinal);
        let remark = Remark::Placed { segment, node: row.node, relation: row.relation };
        Annotation::new(remark, Provenance::from(row.provenance), Standing::Current)
    }
}

impl From<SensedRow> for Annotation {
    fn from(row: SensedRow) -> Annotation {
        let segment = Segment::new(row.message, row.ordinal);
        let sense = Sense::new(row.kind, row.specificity, row.confidence, row.explicitness);
        Annotation::new(Remark::Sensed { segment, sense }, Provenance::from(row.provenance), row.standing)
    }
}

impl From<RelatedRow> for Annotation {
    fn from(row: RelatedRow) -> Annotation {
        let source = Segment::new(row.source_message, row.source_ordinal);
        let target = Segment::new(row.target_message, row.target_ordinal);
        let remark = Remark::Related { source, target, relation: row.relation };
        Annotation::new(remark, Provenance::from(row.provenance), Standing::Current)
    }
}

impl From<ConversationalRow> for Annotation {
    fn from(row: ConversationalRow) -> Annotation {
        let segment = Segment::new(row.message, row.ordinal);
        Annotation::new(Remark::Conversational { segment }, Provenance::from(row.provenance), Standing::Current)
    }
}

impl Annotation {
    pub(crate) fn stored_for(message: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<Annotation>, StoreFailure> {
        let current = Standing::Current;
        let superseded = Standing::Superseded;
        let placed = statements.rows(store_ports::query!(
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
        ))?;
        let sensed = statements.rows(store_ports::query!(
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
        ))?;
        let related = statements.rows(store_ports::query!(
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
        ))?;
        let conversational = statements.rows(store_ports::query!(
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
        ))?;
        let annotations = placed
            .into_iter()
            .map(Annotation::from)
            .chain(sensed.into_iter().map(Annotation::from))
            .chain(related.into_iter().map(Annotation::from))
            .chain(conversational.into_iter().map(Annotation::from));
        Ok(annotations.collect())
    }
}

impl From<&Segment> for Columns {
    fn from(segment: &Segment) -> Columns {
        Columns { message: AsRef::<Uuid>::as_ref(segment).clone(), ordinal: Ordinal::from(segment) }
    }
}

impl Batch for Directions {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let mut annotating = Annotating { provenance: Entered::from(self.provenance), statements };
        let written = self
            .directed
            .iter()
            .map(|directed| directed.written(&mut annotating))
            .collect::<Result<Vec<Written>, Refusal>>()?;
        let recorded = self.file.written(annotating.provenance, annotating.statements)?;
        Ok(Written::total(written.into_iter().chain([recorded])))
    }
}

impl Annotating<'_, '_> {
    pub(super) fn placed(
        &mut self,
        segment: &Segment,
        node: NodeId,
        relation: SpanRelation,
    ) -> Result<Written, Refusal> {
        let Columns { message, ordinal } = Columns::from(segment);
        let provenance = self.provenance;
        Ok(self.statements.written(store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO span_edges (message, ordinal, relation, node, provenance)
VALUES
  (?1, ?2, ?3, ?4, ?5)
"#,
            message: Uuid,
            ordinal: Ordinal,
            relation: SpanRelation,
            node: NodeId,
            provenance: Entered
        ))?)
    }

    pub(super) fn sensed(&mut self, segment: &Segment, sense: Sense) -> Result<Written, Refusal> {
        let Columns { message, ordinal } = Columns::from(segment);
        let kind = sense.kind();
        let specificity = sense.specificity();
        let confidence = sense.confidence();
        let explicitness = sense.explicitness();
        let provenance = self.provenance;
        Ok(self.statements.written(store_ports::query!(
            r#"
INSERT INTO
  senses (
    message,
    ordinal,
    kind,
    specificity,
    confidence,
    explicitness,
    provenance,
    supersedes
  )
VALUES
  (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    (
      SELECT
        earlier.id
      FROM
        senses AS earlier
      WHERE
        earlier.message = ?1
        AND earlier.ordinal = ?2
        AND NOT EXISTS (
          SELECT
            1
          FROM
            senses AS later
          WHERE
            later.supersedes = earlier.id
        )
    )
  )
"#,
            message: Uuid,
            ordinal: Ordinal,
            kind: SenseKind,
            specificity: Specificity,
            confidence: Confidence,
            explicitness: Explicitness,
            provenance: Entered
        ))?)
    }

    pub(super) fn related(
        &mut self,
        source: &Segment,
        target: &Segment,
        relation: SegmentRelation,
    ) -> Result<Written, Refusal> {
        Linking { source, target, relation, provenance: self.provenance }.closed(self.statements)
    }

    pub(super) fn proposed(&mut self, node: &Node<Vague>, parent: NodeId) -> Result<Written, Refusal> {
        let planted = NodeRow::proposed(node, self.statements)?;
        let child = Resolved::read(node.identity(), self.statements)?;
        let parent = Resolved::read(parent, self.statements)?;
        let linked =
            Linked::new(child, NodeRelation::Refines, parent, Provenance::from(self.provenance)).written(self.statements)?;
        Ok(planted.plus(linked))
    }

    pub(super) fn superseded(&mut self, supersession: Supersession) -> Result<Written, Refusal> {
        let [old, by] = supersession.ends();
        let old = Resolved::read(old, self.statements)?;
        let by = Resolved::read(by, self.statements)?;
        Linked::new(by, NodeRelation::Supersedes, old, Provenance::from(self.provenance)).written(self.statements)
    }

    pub(super) fn conversational(&mut self, segment: &Segment) -> Result<Written, Refusal> {
        let Columns { message, ordinal } = Columns::from(segment);
        let provenance = self.provenance;
        Ok(self.statements.written(store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO conversational (message, ordinal, provenance)
VALUES
  (?1, ?2, ?3)
"#,
            message: Uuid,
            ordinal: Ordinal,
            provenance: Entered
        ))?)
    }
}

impl Linking<'_> {
    fn closed(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        match self.relation {
            SegmentRelation::Supersedes => Closing::<SegmentSupersedes>::new(self.source.clone(), self.target.clone())
                .written(statements, |linked| LinkColumns::from(self).written(linked)),
            SegmentRelation::Corrects => Closing::<SegmentCorrects>::new(self.source.clone(), self.target.clone())
                .written(statements, |linked| LinkColumns::from(self).written(linked)),
            SegmentRelation::Answers | SegmentRelation::RefersTo => Ok(LinkColumns::from(self).written(statements)?),
        }
    }
}

impl From<Linking<'_>> for LinkColumns {
    fn from(linking: Linking<'_>) -> LinkColumns {
        LinkColumns {
            source: Columns::from(linking.source),
            relation: linking.relation,
            target: Columns::from(linking.target),
            provenance: linking.provenance,
        }
    }
}

impl LinkColumns {
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let LinkColumns { source, relation, target, provenance } = self;
        let Columns { message: source_message, ordinal: source_ordinal } = source;
        let Columns { message: target_message, ordinal: target_ordinal } = target;
        statements.written(store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO segment_links (
    source_message,
    source_ordinal,
    relation,
    target_message,
    target_ordinal,
    provenance
  )
VALUES
  (?1, ?2, ?3, ?4, ?5, ?6)
"#,
            source_message: Uuid,
            source_ordinal: Ordinal,
            relation: SegmentRelation,
            target_message: Uuid,
            target_ordinal: Ordinal,
            provenance: Entered
        ))
    }
}
