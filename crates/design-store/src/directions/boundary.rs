use design_model::{
    Node, NodeId, NodeRelation, Ordinal, Provenance, Segment, SegmentRelation, Sense, SpanRelation, Standing,
    Supersession,
};
use newtype::{Counter, Newtype};
use store_ports::{Batch, Statements, StoreFailure, Written};
use transcript_ports::Uuid;

use super::Directions;
use crate::acyclic::{Closing, Refusal, SegmentCorrects, SegmentSupersedes};
use crate::annotation::Remark;
use crate::graph::NodeRow;
use crate::{Annotation, Linked};

#[derive(Debug)]
pub(super) struct Annotating<'held, 'run> {
    provenance: Provenance,
    statements: &'held mut Statements<'run>,
}

#[derive(Clone, Copy, Debug)]
struct Linking<'ends> {
    source: &'ends Segment,
    target: &'ends Segment,
    relation: SegmentRelation,
    provenance: &'ends str,
}

#[derive(Debug)]
struct LinkColumns<'provenance> {
    source: Columns,
    relation: &'static str,
    target: Columns,
    provenance: &'provenance str,
}

#[derive(Debug)]
pub(crate) struct Columns {
    message: String,
    ordinal: i64,
}

#[derive(Debug)]
pub(crate) struct PlacedRow {
    message: String,
    ordinal: i64,
    relation: String,
    node: i64,
    provenance: String,
}

#[derive(Debug)]
pub(crate) struct SensedRow {
    message: String,
    ordinal: i64,
    kind: String,
    specificity: String,
    confidence: String,
    explicitness: String,
    provenance: String,
    standing: String,
}

#[derive(Debug)]
pub(crate) struct RelatedRow {
    source_message: String,
    source_ordinal: i64,
    relation: String,
    target_message: String,
    target_ordinal: i64,
    provenance: String,
}

#[derive(Debug)]
pub(crate) struct ConversationalRow {
    message: String,
    ordinal: i64,
    provenance: String,
}

impl Columns {
    fn segment(self) -> Result<Segment, sqlx::Error> {
        let ordinal = usize::try_from(self.ordinal).map_err(|negative| sqlx::Error::Decode(Box::new(negative)))?;
        Ok(Segment::new(Uuid::new(self.message), Ordinal::new(ordinal)))
    }

    pub(crate) fn assigned(self, job: i64, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO job_segments (job, message, ordinal) VALUES (?1, ?2, ?3)",
            job,
            self.message,
            self.ordinal
        ))
    }

    pub(crate) fn rowid(self, statements: &mut Statements<'_>) -> Result<i64, StoreFailure> {
        let rows = statements.rows(sqlx::query!(
            r#"SELECT rowid AS "key!: i64" FROM segments WHERE message = ?1 AND ordinal = ?2"#,
            self.message,
            self.ordinal
        ))?;
        Ok(rows.first().map(|row| row.key).ok_or(sqlx::Error::RowNotFound)?)
    }

    pub(crate) fn covered(message: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<Segment>, StoreFailure> {
        let message = message.to_string();
        let rows = statements.rows(sqlx::query_as!(
            Columns,
            r#"SELECT message AS "message!: String", ordinal AS "ordinal!: i64" FROM (SELECT message, ordinal FROM span_edges WHERE message = ?1 UNION SELECT message, ordinal FROM senses WHERE message = ?1 UNION SELECT message, ordinal FROM conversational WHERE message = ?1 UNION SELECT source_message, source_ordinal FROM segment_links WHERE source_message = ?1 UNION SELECT target_message, target_ordinal FROM segment_links WHERE target_message = ?1) ORDER BY ordinal"#,
            message
        ))?;
        Ok(rows.into_iter().map(Columns::segment).collect::<Result<Vec<Segment>, sqlx::Error>>()?)
    }
}

impl TryFrom<PlacedRow> for Annotation {
    type Error = sqlx::Error;

    fn try_from(row: PlacedRow) -> Result<Annotation, sqlx::Error> {
        let segment = Columns { message: row.message, ordinal: row.ordinal }.segment()?;
        let remark =
            Remark::Placed { segment, node: NodeId::new(row.node), relation: NodeRow::decoded(&row.relation)? };
        Ok(Annotation::new(remark, NodeRow::decoded::<Provenance>(&row.provenance)?, Standing::Current))
    }
}

impl TryFrom<SensedRow> for Annotation {
    type Error = sqlx::Error;

    fn try_from(row: SensedRow) -> Result<Annotation, sqlx::Error> {
        let segment = Columns { message: row.message, ordinal: row.ordinal }.segment()?;
        let sense = Sense::new(
            NodeRow::decoded(&row.kind)?,
            NodeRow::decoded(&row.specificity)?,
            NodeRow::decoded(&row.confidence)?,
            NodeRow::decoded(&row.explicitness)?,
        );
        let remark = Remark::Sensed { segment, sense };
        Ok(Annotation::new(remark, NodeRow::decoded::<Provenance>(&row.provenance)?, NodeRow::decoded(&row.standing)?))
    }
}

impl TryFrom<RelatedRow> for Annotation {
    type Error = sqlx::Error;

    fn try_from(row: RelatedRow) -> Result<Annotation, sqlx::Error> {
        let source = Columns { message: row.source_message, ordinal: row.source_ordinal }.segment()?;
        let target = Columns { message: row.target_message, ordinal: row.target_ordinal }.segment()?;
        let remark = Remark::Related { source, target, relation: NodeRow::decoded(&row.relation)? };
        Ok(Annotation::new(remark, NodeRow::decoded::<Provenance>(&row.provenance)?, Standing::Current))
    }
}

impl TryFrom<ConversationalRow> for Annotation {
    type Error = sqlx::Error;

    fn try_from(row: ConversationalRow) -> Result<Annotation, sqlx::Error> {
        let segment = Columns { message: row.message, ordinal: row.ordinal }.segment()?;
        Ok(Annotation::new(
            Remark::Conversational { segment },
            NodeRow::decoded::<Provenance>(&row.provenance)?,
            Standing::Current,
        ))
    }
}

impl Annotation {
    pub(crate) fn stored_for(message: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<Annotation>, StoreFailure> {
        let message = message.to_string();
        let current = Standing::Current.name();
        let superseded = Standing::Superseded.name();
        let placed = statements.rows(sqlx::query_as!(
            PlacedRow,
            "SELECT message, ordinal, relation, node, provenance FROM span_edges WHERE message = ?1 ORDER BY ordinal, relation, node",
            message
        ))?;
        let sensed = statements.rows(sqlx::query_as!(
            SensedRow,
            r#"SELECT message, ordinal, kind, specificity, confidence, explicitness, provenance, CASE WHEN EXISTS (SELECT 1 FROM senses AS later WHERE later.supersedes = senses.id) THEN ?2 ELSE ?3 END AS "standing!: String" FROM senses WHERE message = ?1 ORDER BY ordinal, id"#,
            message,
            superseded,
            current
        ))?;
        let related = statements.rows(sqlx::query_as!(
            RelatedRow,
            "SELECT source_message, source_ordinal, relation, target_message, target_ordinal, provenance FROM segment_links WHERE source_message = ?1 ORDER BY source_ordinal, relation, target_message, target_ordinal",
            message
        ))?;
        let conversational = statements.rows(sqlx::query_as!(
            ConversationalRow,
            "SELECT message, ordinal, provenance FROM conversational WHERE message = ?1 ORDER BY ordinal",
            message
        ))?;
        let annotations = placed
            .into_iter()
            .map(Annotation::try_from)
            .chain(sensed.into_iter().map(Annotation::try_from))
            .chain(related.into_iter().map(Annotation::try_from))
            .chain(conversational.into_iter().map(Annotation::try_from));
        Ok(annotations.collect::<Result<Vec<Annotation>, sqlx::Error>>()?)
    }
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
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let mut annotating = Annotating { provenance: self.provenance, statements };
        let written = self
            .directed
            .iter()
            .map(|directed| directed.written(&mut annotating))
            .collect::<Result<Vec<Written>, Refusal>>()?;
        let provenance = annotating.provenance.to_string();
        let recorded = self.file.written(&provenance, annotating.statements)?;
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
        let node = node.into_inner();
        let relation = relation.name();
        let provenance = self.provenance.to_string();
        Ok(self.statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO span_edges (message, ordinal, relation, node, provenance) VALUES (?1, ?2, ?3, ?4, ?5)",
            message,
            ordinal,
            relation,
            node,
            provenance
        ))?)
    }

    pub(super) fn sensed(&mut self, segment: &Segment, sense: Sense) -> Result<Written, Refusal> {
        let Columns { message, ordinal } = Columns::from(segment);
        let kind = sense.kind().name();
        let specificity = sense.specificity().name();
        let confidence = sense.confidence().name();
        let explicitness = sense.explicitness().name();
        let provenance = self.provenance.to_string();
        Ok(self.statements.written(sqlx::query!(
            "INSERT INTO senses (message, ordinal, kind, specificity, confidence, explicitness, provenance, supersedes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, (SELECT earlier.id FROM senses AS earlier WHERE earlier.message = ?1 AND earlier.ordinal = ?2 AND NOT EXISTS (SELECT 1 FROM senses AS later WHERE later.supersedes = earlier.id)))",
            message,
            ordinal,
            kind,
            specificity,
            confidence,
            explicitness,
            provenance
        ))?)
    }

    pub(super) fn related(
        &mut self,
        source: &Segment,
        target: &Segment,
        relation: SegmentRelation,
    ) -> Result<Written, Refusal> {
        let provenance = self.provenance.to_string();
        Linking { source, target, relation, provenance: &provenance }.closed(self.statements)
    }

    pub(super) fn proposed(&mut self, node: &Node, parent: NodeId) -> Result<Written, Refusal> {
        let planted = NodeRow::proposed(node, self.statements)?;
        let linked =
            Linked::new(node.identity(), NodeRelation::Refines, parent, self.provenance).written(self.statements)?;
        Ok(planted.plus(linked))
    }

    pub(super) fn superseded(&mut self, supersession: Supersession) -> Result<Written, Refusal> {
        let [old, by] = supersession.ends();
        Linked::new(by, NodeRelation::Supersedes, old, self.provenance).written(self.statements)
    }

    pub(super) fn conversational(&mut self, segment: &Segment) -> Result<Written, Refusal> {
        let Columns { message, ordinal } = Columns::from(segment);
        let provenance = self.provenance.to_string();
        Ok(self.statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO conversational (message, ordinal, provenance) VALUES (?1, ?2, ?3)",
            message,
            ordinal,
            provenance
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

impl<'ends> From<Linking<'ends>> for LinkColumns<'ends> {
    fn from(linking: Linking<'ends>) -> LinkColumns<'ends> {
        LinkColumns {
            source: Columns::from(linking.source),
            relation: linking.relation.name(),
            target: Columns::from(linking.target),
            provenance: linking.provenance,
        }
    }
}

impl LinkColumns<'_> {
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO segment_links (source_message, source_ordinal, relation, target_message, target_ordinal, provenance) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            self.source.message,
            self.source.ordinal,
            self.relation,
            self.target.message,
            self.target.ordinal,
            self.provenance
        ))
    }
}
