use design_model::{Node, NodeId, NodeRelation, Ordinal, Segment, SegmentRelation, Sense, SpanRelation};
use newtype::{Counter, Newtype};
use store_ports::{Batch, Statements, StoreFailure, Written};
use transcript_ports::Uuid;

use super::Directions;
use crate::Linked;
use crate::graph::NodeRow;

#[derive(Debug)]
pub(super) struct Annotating<'held, 'run> {
    statements: &'held mut Statements<'run>,
}

#[derive(Debug)]
struct Columns {
    message: String,
    ordinal: i64,
}

impl From<&Segment> for Columns {
    fn from(segment: &Segment) -> Columns {
        Columns {
            message: AsRef::<Uuid>::as_ref(segment).to_string(),
            ordinal: i64::try_from(Ordinal::from(segment).into_inner()).unwrap_or(i64::MAX),
        }
    }
}

impl Batch for Directions {
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let mut annotating = Annotating { statements };
        let written = self
            .0
            .iter()
            .map(|directed| directed.written(&mut annotating))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

impl Annotating<'_, '_> {
    pub(super) fn placed(
        &mut self,
        segment: &Segment,
        node: NodeId,
        relation: SpanRelation,
    ) -> Result<Written, StoreFailure> {
        let Columns { message, ordinal } = Columns::from(segment);
        let node = node.into_inner();
        let relation = relation.name();
        self.statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO span_edges (message, ordinal, relation, node) VALUES (?1, ?2, ?3, ?4)",
            message,
            ordinal,
            relation,
            node
        ))
    }

    pub(super) fn sensed(&mut self, segment: &Segment, sense: Sense) -> Result<Written, StoreFailure> {
        let Columns { message, ordinal } = Columns::from(segment);
        let kind = sense.kind().name();
        let specificity = sense.specificity().name();
        let confidence = sense.confidence().name();
        let explicitness = sense.explicitness().name();
        self.statements.written(sqlx::query!(
            "INSERT OR REPLACE INTO senses (message, ordinal, kind, specificity, confidence, explicitness) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            message,
            ordinal,
            kind,
            specificity,
            confidence,
            explicitness
        ))
    }

    pub(super) fn related(
        &mut self,
        source: &Segment,
        target: &Segment,
        relation: SegmentRelation,
    ) -> Result<Written, StoreFailure> {
        let source = Columns::from(source);
        let target = Columns::from(target);
        let relation = relation.name();
        self.statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO segment_links (source_message, source_ordinal, relation, target_message, target_ordinal) VALUES (?1, ?2, ?3, ?4, ?5)",
            source.message,
            source.ordinal,
            relation,
            target.message,
            target.ordinal
        ))
    }

    pub(super) fn proposed(&mut self, node: &Node, parent: NodeId) -> Result<Written, StoreFailure> {
        let planted = NodeRow::proposed(node, self.statements)?;
        let linked = Linked::new(node.identity(), NodeRelation::Refines, parent).written(self.statements)?;
        Ok(planted.plus(linked))
    }

    pub(super) fn superseded(&mut self, old: NodeId, by: NodeId) -> Result<Written, StoreFailure> {
        Linked::new(by, NodeRelation::Supersedes, old).written(self.statements)
    }

    pub(super) fn conversational(&mut self, segment: &Segment) -> Result<Written, StoreFailure> {
        let Columns { message, ordinal } = Columns::from(segment);
        self.statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO conversational (message, ordinal) VALUES (?1, ?2)",
            message,
            ordinal
        ))
    }
}
