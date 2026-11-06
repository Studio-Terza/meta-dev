use std::cmp::Ordering;
use std::collections::BTreeMap;

use design_model::Ordinal;
use file_ports::Offset;
use grammar::search::Term;
use newtype::{Arbitrary, Display, Newtype};
use store_ports::{Columns, Statements, StoreFailure, Written};
use stream::{MechanicalLink, Message, Shape};
use transcript_ports::{Authorship, Body, Cursor, Instant, Record, SessionId, Skipped, Uuid};

use crate::Scope;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct ShapeName(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct RelationName(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct AnchorName(String);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SegmentKey(i64);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rank(f64);

pub(crate) trait Row {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure>;
}

#[derive(Debug, Columns)]
pub struct MessageRow {
    uuid: Uuid,
    session: SessionId,
    parent: Option<Uuid>,
    at: Instant,
    author: Authorship,
    body: Body,
}

#[derive(Debug, Columns)]
pub struct SegmentRow {
    message: Uuid,
    ordinal: Ordinal,
    shape: ShapeName,
    body: Body,
}

#[derive(Debug, Columns)]
pub struct LinkRow {
    source: AnchorName,
    relation: RelationName,
    target: AnchorName,
    source_message: Uuid,
    target_message: Uuid,
}

#[derive(Debug, Columns)]
pub struct Hit {
    id: SegmentKey,
    uuid: Uuid,
    author: Authorship,
    at: Instant,
    body: Body,
    rank: Rank,
}

#[derive(Debug, Columns)]
pub(crate) struct RankRow {
    id: SegmentKey,
    rank: Rank,
}

#[derive(Debug, Columns)]
pub(crate) struct EdgeRow {
    session: SessionId,
    uuid: Uuid,
    parent: Option<Uuid>,
}

#[derive(Debug, Columns)]
pub(crate) struct CursorRow {
    session: SessionId,
    read_to: Offset,
}

impl From<&Message> for MessageRow {
    fn from(message: &Message) -> MessageRow {
        let record = message.record();
        MessageRow {
            uuid: record.uuid().clone(),
            session: record.session().clone(),
            parent: record.parent().cloned(),
            at: record.stamp().clone(),
            author: record.author(),
            body: record.text().clone(),
        }
    }
}

impl From<MessageRow> for Record {
    fn from(row: MessageRow) -> Record {
        Skipped::new(row.uuid, row.parent).said(row.session, row.at, row.author, row.body)
    }
}

impl From<EdgeRow> for Skipped {
    fn from(row: EdgeRow) -> Skipped {
        Skipped::new(row.uuid, row.parent)
    }
}

impl From<&SegmentRow> for Body {
    fn from(segment: &SegmentRow) -> Body {
        segment.body.clone()
    }
}

impl From<&Hit> for Body {
    fn from(hit: &Hit) -> Body {
        hit.body.clone()
    }
}

impl From<&MechanicalLink> for LinkRow {
    fn from(link: &MechanicalLink) -> LinkRow {
        let [source, target] = link.anchors();
        LinkRow {
            source: AnchorName::new(source.to_string()),
            relation: RelationName::new(link.relation().to_string()),
            target: AnchorName::new(target.to_string()),
            source_message: source.message().clone(),
            target_message: target.message().clone(),
        }
    }
}

impl MessageRow {
    pub(crate) fn named(uuid: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<MessageRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            MessageRow,
            "SELECT uuid, session, parent, at, author, body FROM messages WHERE uuid = ?1",
            uuid: Uuid
        ))
    }

    pub(crate) fn earlier(session: &SessionId, statements: &mut Statements<'_>) -> Result<Vec<Record>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            MessageRow,
            "SELECT messages.uuid, messages.session, messages.parent, messages.at, messages.author, messages.body FROM edges JOIN messages ON messages.uuid = edges.uuid WHERE edges.session = ?1 ORDER BY edges.rowid",
            session: SessionId
        ))?;
        Ok(rows.into_iter().map(Record::from).collect())
    }

    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.uuid.clone()
    }

    #[must_use]
    pub fn session(&self) -> SessionId {
        self.session.clone()
    }

    #[must_use]
    pub fn stamp(&self) -> Instant {
        self.at.clone()
    }

    #[must_use]
    pub fn author(&self) -> Authorship {
        self.author
    }
}

impl SegmentRow {
    pub(crate) fn within(message: &Message) -> Vec<SegmentRow> {
        let uuid = message.record().uuid();
        message
            .segments()
            .iter()
            .map(|segment| SegmentRow {
                message: uuid.clone(),
                ordinal: Ordinal::new(segment.identity().ordinal().into_inner()),
                shape: ShapeName::new(Shape::from(segment.form()).to_string()),
                body: segment.text(message),
            })
            .collect()
    }

    pub(crate) fn stored_for(uuid: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<SegmentRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            SegmentRow,
            "SELECT message, ordinal, shape, body FROM segments WHERE message = ?1 ORDER BY ordinal",
            uuid: Uuid
        ))
    }

    #[must_use]
    pub fn ordinal(&self) -> Ordinal {
        self.ordinal
    }

    #[must_use]
    pub fn shape(&self) -> ShapeName {
        self.shape.clone()
    }
}

impl LinkRow {
    pub(crate) fn touching(uuid: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<LinkRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            LinkRow,
            "SELECT source, relation, target, source_message, target_message FROM mechanical_links WHERE ?1 IN (source_message, target_message) ORDER BY rowid",
            uuid: Uuid
        ))
    }

    #[must_use]
    pub fn anchors(&self) -> [AnchorName; 2] {
        [self.source.clone(), self.target.clone()]
    }

    #[must_use]
    pub fn relation(&self) -> RelationName {
        self.relation.clone()
    }
}

impl Hit {
    pub(crate) fn matching(
        term: &Term,
        scope: Scope,
        statements: &mut Statements<'_>,
    ) -> Result<Vec<Hit>, StoreFailure> {
        let only = scope.only();
        statements.rows(store_ports::query!(
            Hit,
            r#"SELECT segments.rowid AS "id!: i64", messages.uuid, messages.author, messages.at, segments.body, segments_fts.rank AS "rank!: f64" FROM segments_fts JOIN segments ON segments.rowid = segments_fts.rowid JOIN messages ON messages.uuid = segments.message WHERE segments_fts MATCH ?1 AND (?2 IS NULL OR messages.author = ?2)"#,
            term: Term,
            only: Option<Authorship>
        ))
    }

    pub(crate) fn within(self, rest: &[BTreeMap<SegmentKey, Rank>]) -> Option<Hit> {
        rest.iter()
            .map(|ranks| ranks.get(&self.id).map(|rank| rank.into_inner()))
            .sum::<Option<f64>>()
            .map(|more| Hit { rank: Rank::new(self.rank.into_inner() + more), ..self })
    }

    pub(crate) fn ranking(&self, other: &Hit) -> Ordering {
        self.rank.into_inner().total_cmp(&other.rank.into_inner()).then(self.id.cmp(&other.id))
    }

    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.uuid.clone()
    }

    #[must_use]
    pub fn author(&self) -> Authorship {
        self.author
    }

    #[must_use]
    pub fn stamp(&self) -> Instant {
        self.at.clone()
    }
}

impl RankRow {
    pub(crate) fn matching(term: &Term, statements: &mut Statements<'_>) -> Result<Vec<RankRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            RankRow,
            r#"SELECT rowid AS "id!: i64", rank AS "rank!: f64" FROM segments_fts WHERE segments_fts MATCH ?1"#,
            term: Term
        ))
    }

    pub(crate) fn ranked(rows: Vec<RankRow>) -> BTreeMap<SegmentKey, Rank> {
        rows.into_iter().map(|row| (row.id, row.rank)).collect()
    }
}

impl EdgeRow {
    pub(crate) fn said(session: &SessionId, message: &Message) -> EdgeRow {
        let record = message.record();
        EdgeRow { session: session.clone(), uuid: record.uuid().clone(), parent: record.parent().cloned() }
    }

    pub(crate) fn skipped(session: &SessionId, skipped: &Skipped) -> EdgeRow {
        EdgeRow { session: session.clone(), uuid: skipped.uuid().clone(), parent: skipped.parent().cloned() }
    }

    pub(crate) fn passing(session: &SessionId, statements: &mut Statements<'_>) -> Result<Vec<Skipped>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            EdgeRow,
            "SELECT session, uuid, parent FROM edges WHERE session = ?1 AND uuid NOT IN (SELECT uuid FROM messages) ORDER BY rowid",
            session: SessionId
        ))?;
        Ok(rows.into_iter().map(Skipped::from).collect())
    }
}

impl CursorRow {
    pub(crate) fn reached(session: &SessionId, cursor: Cursor) -> CursorRow {
        CursorRow { session: session.clone(), read_to: cursor.into_inner() }
    }

    pub(crate) fn all(statements: &mut Statements<'_>) -> Result<Vec<CursorRow>, StoreFailure> {
        statements.rows(store_ports::query!(CursorRow, "SELECT session, read_to FROM cursors"))
    }

    pub(crate) fn session(&self) -> SessionId {
        self.session.clone()
    }

    pub(crate) fn cursor(&self) -> Cursor {
        Cursor::new(self.read_to)
    }
}

impl Row for MessageRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let MessageRow { uuid, session, parent, at, author, body } = self;
        statements.written(store_ports::query!(
            "INSERT OR IGNORE INTO messages (uuid, session, parent, at, author, body) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            uuid: Uuid,
            session: SessionId,
            parent: Option<Uuid>,
            at: Instant,
            author: Authorship,
            body: Body
        ))
    }
}

impl Row for SegmentRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let SegmentRow { message, ordinal, shape, body } = self;
        statements.written(store_ports::query!(
            "INSERT OR IGNORE INTO segments (message, ordinal, shape, body) VALUES (?1, ?2, ?3, ?4)",
            message: Uuid,
            ordinal: Ordinal,
            shape: ShapeName,
            body: Body
        ))
    }
}

impl Row for LinkRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let LinkRow { source, relation, target, source_message, target_message } = self;
        statements.written(store_ports::query!(
            "INSERT OR IGNORE INTO mechanical_links (source, relation, target, source_message, target_message) VALUES (?1, ?2, ?3, ?4, ?5)",
            source: AnchorName,
            relation: RelationName,
            target: AnchorName,
            source_message: Uuid,
            target_message: Uuid
        ))
    }
}

impl Row for EdgeRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let EdgeRow { session, uuid, parent } = self;
        statements.written(store_ports::query!(
            "INSERT OR IGNORE INTO edges (session, uuid, parent) VALUES (?1, ?2, ?3)",
            session: SessionId,
            uuid: Uuid,
            parent: Option<Uuid>
        ))
    }
}

impl Row for CursorRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let CursorRow { session, read_to } = self;
        statements.written(store_ports::query!(
            "INSERT INTO cursors (session, read_to) VALUES (?1, ?2) ON CONFLICT (session) DO UPDATE SET read_to = excluded.read_to WHERE read_to <> excluded.read_to",
            session: SessionId,
            read_to: Offset
        ))
    }
}
