use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::iter::Sum;

use design_model::{Ordinal, Segment};
use dispatch_ports::Reply;
use file_ports::Offset;
use grammar::search::Term;
use newtype::{Arbitrary, Display, Newtype, Wrapping};
use store_ports::{Columns, Statements, StoreFailure, Written};
use stream::{Anchor, MechanicalLink, Message, Shape};
use transcript_ports::{Authorship, Body, Cursor, Instant, Record, SessionId, Skipped, Uuid};

use crate::Scope;
use crate::acyclic::Keyed;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct ShapeName(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct RelationName(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Account(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Excerpt(String);

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

#[derive(Debug, PartialEq, Columns)]
pub struct SegmentRow {
    message: Uuid,
    ordinal: Ordinal,
    shape: ShapeName,
    body: Body,
}

#[derive(Debug, Columns)]
pub struct LinkRow {
    source: Anchor,
    relation: RelationName,
    target: Anchor,
    source_message: Uuid,
    target_message: Uuid,
}

#[derive(Debug, Columns)]
pub struct Hit {
    id: Keyed,
    uuid: Uuid,
    ordinal: Ordinal,
    author: Authorship,
    at: Instant,
    body: Body,
    rank: Rank,
}

#[derive(Debug, Columns)]
pub(crate) struct RankRow {
    id: Keyed,
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
            source: source.clone(),
            relation: RelationName::new(link.relation().to_string()),
            target: target.clone(),
            source_message: source.message().clone(),
            target_message: target.message().clone(),
        }
    }
}

impl MessageRow {
    pub(crate) fn named(uuid: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<MessageRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            MessageRow,
            r#"
SELECT
  uuid,
  session,
  parent,
  at,
  author,
  body
FROM
  messages
WHERE
  uuid = ?1
"#,
            uuid: Uuid
        ))
    }

    pub(crate) fn every(statements: &mut Statements<'_>) -> Result<Vec<Message>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            MessageRow,
            r#"
SELECT
  uuid,
  session,
  parent,
  at,
  author,
  body
FROM
  messages
"#
        ))?;
        Ok(rows.into_iter().map(Record::from).map(Message::from).collect())
    }

    pub(crate) fn earlier(session: &SessionId, statements: &mut Statements<'_>) -> Result<Vec<Record>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            MessageRow,
            r#"
SELECT
  messages.uuid,
  messages.session,
  messages.parent,
  messages.at,
  messages.author,
  messages.body
FROM
  edges
  JOIN messages ON messages.uuid = edges.uuid
WHERE
  edges.session = ?1
ORDER BY
  edges.rowid
"#,
            session: SessionId
        ))?;
        Ok(rows.into_iter().map(Record::from).collect())
    }
}

impl SegmentRow {
    pub(crate) fn within(message: &Message) -> Vec<SegmentRow> {
        let uuid = message.record().uuid();
        message
            .segments()
            .iter()
            .filter_map(|segment| {
                segment.text(message).ok().map(|body| SegmentRow {
                    message: uuid.clone(),
                    ordinal: Ordinal::new(segment.identity().ordinal().into_inner()),
                    shape: ShapeName::new(Shape::from(segment.form()).to_string()),
                    body,
                })
            })
            .collect()
    }

    pub(crate) fn stored_for(uuid: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<SegmentRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            SegmentRow,
            r#"
SELECT
  message,
  ordinal,
  shape,
  body
FROM
  segments
WHERE
  message = ?1
ORDER BY
  ordinal
"#,
            uuid: Uuid
        ))
    }

    pub(crate) fn every(statements: &mut Statements<'_>) -> Result<Vec<SegmentRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            SegmentRow,
            r#"
SELECT
  message,
  ordinal,
  shape,
  body
FROM
  segments
ORDER BY
  message,
  ordinal
"#
        ))
    }

    pub(crate) fn released(&self, statements: &mut Statements<'_>) -> Result<Option<Written>, StoreFailure> {
        let SegmentRow { message, ordinal, .. } = self;
        statements.written_unless_referenced(store_ports::query!(
            r#"
DELETE FROM
  segments
WHERE
  message = ?1
  AND ordinal = ?2
"#,
            message: Uuid,
            ordinal: Ordinal
        ))
    }

    pub(crate) fn segment(&self) -> Segment {
        Segment::new(self.message.clone(), self.ordinal)
    }

    #[must_use]
    pub fn ordinal(&self) -> Ordinal {
        self.ordinal
    }
}

impl LinkRow {
    pub(crate) fn touching(uuid: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<LinkRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            LinkRow,
            r#"
SELECT
  source,
  relation,
  target,
  source_message,
  target_message
FROM
  mechanical_links
WHERE
  ?1 IN (source_message, target_message)
ORDER BY
  rowid
"#,
            uuid: Uuid
        ))
    }

    fn ordinal(anchor: &Anchor) -> Option<Ordinal> {
        match anchor {
            Anchor::Message(_) => None,
            Anchor::Segment(id) => Some(Ordinal::new(id.ordinal().into_inner())),
        }
    }

    #[must_use]
    pub fn anchors(&self) -> [Anchor; 2] {
        [self.source.clone(), self.target.clone()]
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
            r#"
SELECT
  segments.rowid AS "id!: i64",
  messages.uuid,
  segments.ordinal,
  messages.author,
  messages.at,
  segments.body,
  segments_fts.rank AS "rank!: f64"
FROM
  segments_fts
  JOIN segments ON segments.rowid = segments_fts.rowid
  JOIN messages ON messages.uuid = segments.message
WHERE
  segments_fts MATCH ?1
  AND (
    ?2 IS NULL
    OR messages.author = ?2
  )
"#,
            term: Term,
            only: Option<Authorship>
        ))
    }

    pub(crate) fn within(self, rest: &[BTreeMap<Keyed, Rank>]) -> Option<Hit> {
        rest.iter()
            .map(|ranks| ranks.get(&self.id).copied())
            .chain([Some(self.rank)])
            .sum::<Option<Rank>>()
            .map(|rank| Hit { rank, ..self })
    }

    pub(crate) fn ranking(&self, other: &Hit) -> Ordering {
        self.rank.ordered(other.rank).then(self.id.cmp(&other.id))
    }
}

impl Rank {
    fn ordered(self, other: Rank) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Sum for Rank {
    fn sum<Ranks: Iterator<Item = Rank>>(ranks: Ranks) -> Rank {
        Rank(ranks.map(|Rank(rank)| rank).sum())
    }
}

impl RankRow {
    pub(crate) fn matching(term: &Term, statements: &mut Statements<'_>) -> Result<Vec<RankRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            RankRow,
            r#"
SELECT
  rowid AS "id!: i64",
  rank AS "rank!: f64"
FROM
  segments_fts
WHERE
  segments_fts MATCH ?1
"#,
            term: Term
        ))
    }

    pub(crate) fn ranked(rows: Vec<RankRow>) -> BTreeMap<Keyed, Rank> {
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
            r#"
SELECT
  session,
  uuid,
  parent
FROM
  edges
WHERE
  session = ?1
  AND uuid NOT IN (
    SELECT
      uuid
    FROM
      messages
  )
ORDER BY
  rowid
"#,
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
        statements.rows(store_ports::query!(
            CursorRow,
            r#"
SELECT
  session,
  read_to
FROM
  cursors
"#
        ))
    }

    pub(crate) fn session(&self) -> &SessionId {
        &self.session
    }
}

impl Row for MessageRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let MessageRow { uuid, session, parent, at, author, body } = self;
        statements.written(store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO messages (uuid, session, parent, at, author, body)
VALUES
  (?1, ?2, ?3, ?4, ?5, ?6)
"#,
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
            r#"
INSERT
  OR IGNORE INTO segments (message, ordinal, shape, body)
VALUES
  (?1, ?2, ?3, ?4)
"#,
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
        let source_ordinal = LinkRow::ordinal(source);
        let target_ordinal = LinkRow::ordinal(target);
        statements.written(store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO mechanical_links (
    source,
    relation,
    target,
    source_message,
    source_ordinal,
    target_message,
    target_ordinal
  )
VALUES
  (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#,
            source: Anchor,
            relation: RelationName,
            target: Anchor,
            source_message: Uuid,
            source_ordinal: Option<Ordinal>,
            target_message: Uuid,
            target_ordinal: Option<Ordinal>
        ))
    }
}

impl Row for EdgeRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let EdgeRow { session, uuid, parent } = self;
        statements.written(store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO edges (session, uuid, parent)
VALUES
  (?1, ?2, ?3)
"#,
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
            r#"
INSERT INTO
  cursors (session, read_to)
VALUES
  (?1, ?2)
ON CONFLICT
  (session)
DO UPDATE SET
  read_to = excluded.read_to
WHERE
  read_to <> excluded.read_to
"#,
            session: SessionId,
            read_to: Offset
        ))
    }
}

impl From<&CursorRow> for Cursor {
    fn from(row: &CursorRow) -> Cursor {
        Cursor::new(row.read_to)
    }
}

impl AsRef<Uuid> for MessageRow {
    fn as_ref(&self) -> &Uuid {
        &self.uuid
    }
}

impl AsRef<SessionId> for MessageRow {
    fn as_ref(&self) -> &SessionId {
        &self.session
    }
}

impl AsRef<Instant> for MessageRow {
    fn as_ref(&self) -> &Instant {
        &self.at
    }
}

impl AsRef<Authorship> for MessageRow {
    fn as_ref(&self) -> &Authorship {
        &self.author
    }
}

impl AsRef<ShapeName> for SegmentRow {
    fn as_ref(&self) -> &ShapeName {
        &self.shape
    }
}

impl AsRef<RelationName> for LinkRow {
    fn as_ref(&self) -> &RelationName {
        &self.relation
    }
}

impl From<&Hit> for Segment {
    fn from(hit: &Hit) -> Segment {
        Segment::new(hit.uuid.clone(), hit.ordinal)
    }
}

impl AsRef<Authorship> for Hit {
    fn as_ref(&self) -> &Authorship {
        &self.author
    }
}

impl AsRef<Instant> for Hit {
    fn as_ref(&self) -> &Instant {
        &self.at
    }
}

impl Account {
    #[must_use]
    pub fn told(told: &impl fmt::Display) -> Account {
        Account(told.to_string())
    }
}

impl Excerpt {
    const BYTES: usize = 16 * 1024;
}

impl From<&Reply> for Excerpt {
    fn from(reply: &Reply) -> Excerpt {
        let told = reply.clone().into_inner();
        let kept = told.get(..told.floor_char_boundary(Excerpt::BYTES)).unwrap_or_default();
        Excerpt(kept.to_owned())
    }
}
