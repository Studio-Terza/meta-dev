use design_model::{
    Entered, Node, NodeId, NodeRelation, Ordinal, Provenance, Segment, SegmentRelation, Sense, SpanRelation, Standing,
    Supersession,
};
use newtype::{Counter, Newtype};
use store_ports::{Batch, Statements, StoreFailure, Written};
use transcript_ports::Uuid;

use super::{Directed, Directing, Directions};
use crate::graph::NodeRow;
use crate::{Annotation, Linked};

#[derive(Debug)]
pub(super) struct Annotating<'held, 'run> {
    provenance: Provenance,
    statements: &'held mut Statements<'run>,
}

#[derive(Debug)]
struct Columns {
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

    fn provenance(text: &str) -> Result<Provenance, sqlx::Error> {
        Ok(Provenance::from(NodeRow::decoded::<Entered>(text)?))
    }
}

impl TryFrom<PlacedRow> for Annotation {
    type Error = sqlx::Error;

    fn try_from(row: PlacedRow) -> Result<Annotation, sqlx::Error> {
        let segment = Columns { message: row.message, ordinal: row.ordinal }.segment()?;
        let directed =
            Directed::Placed { segment, node: NodeId::new(row.node), relation: NodeRow::decoded(&row.relation)? };
        Ok(Annotation::new(directed, Columns::provenance(&row.provenance)?, Standing::Current))
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
        let directed = Directed::Sensed { segment, sense };
        Ok(Annotation::new(directed, Columns::provenance(&row.provenance)?, NodeRow::decoded(&row.standing)?))
    }
}

impl TryFrom<RelatedRow> for Annotation {
    type Error = sqlx::Error;

    fn try_from(row: RelatedRow) -> Result<Annotation, sqlx::Error> {
        let source = Columns { message: row.source_message, ordinal: row.source_ordinal }.segment()?;
        let target = Columns { message: row.target_message, ordinal: row.target_ordinal }.segment()?;
        let directed = Directed::Related { source, target, relation: NodeRow::decoded(&row.relation)? };
        Ok(Annotation::new(directed, Columns::provenance(&row.provenance)?, Standing::Current))
    }
}

impl TryFrom<ConversationalRow> for Annotation {
    type Error = sqlx::Error;

    fn try_from(row: ConversationalRow) -> Result<Annotation, sqlx::Error> {
        let segment = Columns { message: row.message, ordinal: row.ordinal }.segment()?;
        Ok(Annotation::new(
            Directed::Conversational { segment },
            Columns::provenance(&row.provenance)?,
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
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let mut annotating = Annotating { provenance: self.provenance, statements };
        let written = self
            .directed
            .iter()
            .map(|directed| directed.visited(&mut annotating))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

impl Annotating<'_, '_> {
    fn provenance(&self) -> &'static str {
        Entered::from(self.provenance).name()
    }
}

impl Directing for &mut Annotating<'_, '_> {
    type Output = Result<Written, StoreFailure>;

    fn placed(self, segment: &Segment, node: NodeId, relation: SpanRelation) -> Result<Written, StoreFailure> {
        let Columns { message, ordinal } = Columns::from(segment);
        let node = node.into_inner();
        let relation = relation.name();
        let provenance = self.provenance();
        self.statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO span_edges (message, ordinal, relation, node, provenance) VALUES (?1, ?2, ?3, ?4, ?5)",
            message,
            ordinal,
            relation,
            node,
            provenance
        ))
    }

    fn sensed(self, segment: &Segment, sense: Sense) -> Result<Written, StoreFailure> {
        let Columns { message, ordinal } = Columns::from(segment);
        let kind = sense.kind().name();
        let specificity = sense.specificity().name();
        let confidence = sense.confidence().name();
        let explicitness = sense.explicitness().name();
        let provenance = self.provenance();
        self.statements.written(sqlx::query!(
            "INSERT INTO senses (message, ordinal, kind, specificity, confidence, explicitness, provenance, supersedes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, (SELECT earlier.id FROM senses AS earlier WHERE earlier.message = ?1 AND earlier.ordinal = ?2 AND NOT EXISTS (SELECT 1 FROM senses AS later WHERE later.supersedes = earlier.id)))",
            message,
            ordinal,
            kind,
            specificity,
            confidence,
            explicitness,
            provenance
        ))
    }

    fn related(self, source: &Segment, target: &Segment, relation: SegmentRelation) -> Result<Written, StoreFailure> {
        let source = Columns::from(source);
        let target = Columns::from(target);
        let relation = relation.name();
        let provenance = self.provenance();
        self.statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO segment_links (source_message, source_ordinal, relation, target_message, target_ordinal, provenance) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            source.message,
            source.ordinal,
            relation,
            target.message,
            target.ordinal,
            provenance
        ))
    }

    fn proposed(self, node: &Node, parent: NodeId) -> Result<Written, StoreFailure> {
        let planted = NodeRow::proposed(node, self.statements)?;
        let linked =
            Linked::new(node.identity(), NodeRelation::Refines, parent, self.provenance).written(self.statements)?;
        Ok(planted.plus(linked))
    }

    fn superseded(self, supersession: Supersession) -> Result<Written, StoreFailure> {
        let [old, by] = supersession.ends();
        Linked::new(by, NodeRelation::Supersedes, old, self.provenance).written(self.statements)
    }

    fn conversational(self, segment: &Segment) -> Result<Written, StoreFailure> {
        let Columns { message, ordinal } = Columns::from(segment);
        let provenance = self.provenance();
        self.statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO conversational (message, ordinal, provenance) VALUES (?1, ?2, ?3)",
            message,
            ordinal,
            provenance
        ))
    }
}
