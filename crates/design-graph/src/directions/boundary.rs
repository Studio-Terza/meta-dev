use design_model::{
    Confidence, Explicitness, Node, NodeId, NodeRelation, Ordinal, Provenance, Segment, SegmentRelation, Sense,
    SenseKind, SpanRelation, Specificity, Supersession, Vague,
};
use store_ports::{Batch, Columns as Row, Counts, Done, Execute, StoreFailure};
use transcript_ports::Uuid;

use super::Directions;
use crate::Refusal;
use crate::graph::{NodeRow, Resolved};
use design_closure::{Closing, SegmentCorrects, SegmentSupersedes};
use design_jobs::JobState;

use crate::Linked;

#[derive(Debug)]
pub(crate) struct Annotating<'held, E: Execute> {
    provenance: Provenance,
    statements: &'held mut E,
}

#[derive(Clone, Copy, Debug)]
struct Linking<'ends> {
    source: &'ends Segment,
    target: &'ends Segment,
    relation: SegmentRelation,
    provenance: Provenance,
}

#[derive(Debug)]
struct LinkColumns {
    source: Columns,
    relation: SegmentRelation,
    target: Columns,
    provenance: Provenance,
}

#[derive(Debug, Row)]
struct Columns {
    message: Uuid,
    ordinal: Ordinal,
}

impl From<&Segment> for Columns {
    fn from(segment: &Segment) -> Columns {
        Columns { message: AsRef::<Uuid>::as_ref(segment).clone(), ordinal: Ordinal::from(segment) }
    }
}

impl Batch for Directions {
    type Failure = Refusal;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), Refusal> {
        self.counted(statements)
    }
}

impl Counts for Directions {
    fn counted<D: Done>(self, statements: &mut impl Execute) -> Result<D, Refusal> {
        let mut annotating = Annotating { provenance: self.provenance, statements };
        let written = self
            .directed
            .iter()
            .map(|directed| directed.written(&mut annotating))
            .collect::<Result<Vec<D>, Refusal>>()?;
        let recorded = self.file.written(annotating.provenance, annotating.statements)?;
        let closed = self.remit.paid(annotating.statements)?.closed(JobState::Applied, annotating.statements)?;
        let unflailed = Annotating::unflailed(annotating.statements)?;
        Ok(Done::total(written.into_iter().chain([recorded, closed, unflailed])))
    }
}

impl<'held, E: Execute> Annotating<'held, E> {
    pub(crate) fn new(provenance: Provenance, statements: &'held mut E) -> Annotating<'held, E> {
        Annotating { provenance, statements }
    }

    pub(crate) fn unflailed<D: Done>(statements: &mut E) -> Result<D, StoreFailure> {
        statements.done(store_ports::query!(
            r#"
DELETE FROM
  flailed
WHERE
  EXISTS (
    SELECT
      1
    FROM
      structured
    WHERE
      structured.message = flailed.message
      AND structured.ordinal = flailed.ordinal
  )
"#
        ))
    }

    pub(super) fn placed<D: Done>(
        &mut self,
        segment: &Segment,
        node: NodeId,
        relation: SpanRelation,
    ) -> Result<D, Refusal> {
        let Columns { message, ordinal } = Columns::from(segment);
        let provenance = self.provenance;
        Ok(self.statements.done(store_ports::query!(
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
            provenance: Provenance
        ))?)
    }

    pub(super) fn sensed<D: Done>(&mut self, segment: &Segment, sense: Sense) -> Result<D, Refusal> {
        let Columns { message, ordinal } = Columns::from(segment);
        let kind = sense.kind();
        let specificity = sense.specificity();
        let confidence = sense.confidence();
        let explicitness = sense.explicitness();
        let provenance = self.provenance;
        Ok(self.statements.done(store_ports::query!(
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
            provenance: Provenance
        ))?)
    }

    pub(super) fn related<D: Done>(
        &mut self,
        source: &Segment,
        target: &Segment,
        relation: SegmentRelation,
    ) -> Result<D, Refusal> {
        Linking { source, target, relation, provenance: self.provenance }.closed(self.statements)
    }

    pub(super) fn proposed<D: Done>(&mut self, node: &Node<Vague>, parent: NodeId) -> Result<D, Refusal> {
        let planted = NodeRow::proposed(node, self.statements)?;
        let child = Resolved::read(node.identity(), self.statements)?;
        let refined = Resolved::read(parent, self.statements)?;
        let linked = Linked::new(child, NodeRelation::Refines, refined, self.provenance).linked(self.statements)?;
        Ok(Done::joined(planted, linked))
    }

    pub(super) fn superseded<D: Done>(&mut self, supersession: Supersession) -> Result<D, Refusal> {
        let [old, by] = supersession.ends();
        let replaced = Resolved::read(old, self.statements)?;
        let replacing = Resolved::read(by, self.statements)?;
        Linked::new(replacing, NodeRelation::Supersedes, replaced, self.provenance).linked(self.statements)
    }

    pub(super) fn cited<D: Done>(&mut self, node: NodeId, uuid: &Uuid) -> Result<D, Refusal> {
        let message = uuid.clone();
        let provenance = self.provenance;
        Ok(self.statements.done(store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO node_cites (node, message, provenance)
VALUES
  (?1, ?2, ?3)
"#,
            node: NodeId,
            message: Uuid,
            provenance: Provenance
        ))?)
    }

    pub(super) fn conversational<D: Done>(&mut self, segment: &Segment) -> Result<D, Refusal> {
        let Columns { message, ordinal } = Columns::from(segment);
        let provenance = self.provenance;
        Ok(self.statements.done(store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO conversational (message, ordinal, provenance)
VALUES
  (?1, ?2, ?3)
"#,
            message: Uuid,
            ordinal: Ordinal,
            provenance: Provenance
        ))?)
    }
}

impl Linking<'_> {
    fn closed<D: Done>(self, statements: &mut impl Execute) -> Result<D, Refusal> {
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
    fn written<D: Done>(self, statements: &mut impl Execute) -> Result<D, StoreFailure> {
        let LinkColumns { source, relation, target, provenance } = self;
        let Columns { message: source_message, ordinal: source_ordinal } = source;
        let Columns { message: target_message, ordinal: target_ordinal } = target;
        statements.done(store_ports::query!(
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
            provenance: Provenance
        ))
    }
}

impl From<Columns> for Segment {
    fn from(columns: Columns) -> Segment {
        Segment::new(columns.message, columns.ordinal)
    }
}
