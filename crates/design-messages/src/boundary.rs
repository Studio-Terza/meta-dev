pub(crate) mod ignored;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::iter::Sum;

use design_model::{Ordinal, Segment};
use file_ports::Offset;
use grammar_search::Term;
use newtype::{Arbitrary, Newtype, Wrapping};
use store_ports::Record as _;
use store_ports::{Columns, Deletion, Execute, Prepared, StoreFailure, Written};
use stream::{Anchor, Anchors, MechanicalLink, Message, Relation, SegmentId, Shape};
use transcript_ports::{Authorship, Body, Cursor, Instant, Record, SessionId, Skipped, Uuid};

use crate::boundary::ignored::Ignored;
use crate::{Said, Scope};
use design_closure::Keyed;
use invariants::Ignoring;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rank(f64);

pub(crate) trait Row {
    fn inserted(&self) -> Prepared<()>;
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

#[derive(Clone, Debug, PartialEq, Columns)]
pub struct SegmentRow {
    message: Uuid,
    ordinal: Ordinal,
    shape: Shape,
    body: Body,
}

#[derive(Clone, Debug, Columns)]
pub struct LinkRow {
    source: Anchor,
    relation: Relation,
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

#[derive(Clone, Debug, PartialEq, Eq, Columns, store_ports::Record)]
#[record(table = "cursors")]
pub struct CursorRow {
    #[record(key)]
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

impl From<MessageRow> for Said {
    fn from(row: MessageRow) -> Said {
        Said::new(row.uuid, row.at, row.body)
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
            relation: link.relation(),
            target: target.clone(),
            source_message: source.message().clone(),
            target_message: target.message().clone(),
        }
    }
}

impl MessageRow {
    pub(crate) fn named(uuid: &Uuid) -> Prepared<MessageRow> {
        store_ports::query!(
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
        )
    }

    pub(crate) fn last_in(
        session: &SessionId,
        author: Authorship,
        statements: &mut impl Execute,
    ) -> Result<Vec<MessageRow>, StoreFailure> {
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
  session = ?1
  AND author = ?2
ORDER BY
  at DESC
LIMIT
  1
"#,
            session: SessionId,
            author: Authorship
        ))
    }

    pub(crate) fn owned(statements: &mut impl Execute) -> Result<Vec<Said>, StoreFailure> {
        let owned: BTreeMap<(Instant, Uuid), MessageRow> = MessageRow::all(statements)?
            .into_iter()
            .filter(|row| row.author == Authorship::Owner)
            .map(|row| ((row.at.clone(), row.uuid.clone()), row))
            .collect();
        Ok(owned.into_values().map(Said::from).collect())
    }

    pub(crate) fn every(statements: &mut impl Execute) -> Result<Vec<Message>, StoreFailure> {
        Ok(MessageRow::all(statements)?.into_iter().map(Record::from).map(Message::from).collect())
    }

    fn all(statements: &mut impl Execute) -> Result<Vec<MessageRow>, StoreFailure> {
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
"#
        ))
    }

    pub(crate) fn earlier(session: &SessionId, statements: &mut impl Execute) -> Result<Vec<Record>, StoreFailure> {
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
                segment.text(message).ignoring(Ignored::Unsliced).map(|body| SegmentRow {
                    message: uuid.clone(),
                    ordinal: Ordinal::new(segment.identity().ordinal().into_inner()),
                    shape: Shape::from(segment.form()),
                    body,
                })
            })
            .collect()
    }

    pub(crate) fn stored_for(uuid: &Uuid) -> Prepared<SegmentRow> {
        store_ports::query!(
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
        )
    }

    pub(crate) fn every(statements: &mut impl Execute) -> Result<Vec<SegmentRow>, StoreFailure> {
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

    pub(crate) fn deleted(&self) -> Deletion {
        let SegmentRow { message, ordinal, .. } = self;
        Deletion::from(store_ports::query!(
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
    pub(crate) fn touching(uuid: &Uuid, statements: &mut impl Execute) -> Result<Vec<LinkRow>, StoreFailure> {
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

    #[must_use]
    pub fn anchors(&self) -> [Anchor; 2] {
        [self.source.clone(), self.target.clone()]
    }

    pub(crate) fn segments(&self) -> Vec<Segment> {
        [&self.source, &self.target].into_iter().filter_map(|anchor| anchor.visited(Held)).collect()
    }

    pub(crate) fn unchecked(statements: &mut impl Execute) -> Result<Vec<LinkRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            LinkRow,
            r#"
SELECT
  mechanical_links.source,
  mechanical_links.relation,
  mechanical_links.target,
  mechanical_links.source_message,
  mechanical_links.target_message
FROM
  pragma_foreign_key_check('mechanical_links') AS unchecked
  JOIN mechanical_links ON mechanical_links.rowid = unchecked.rowid
GROUP BY
  mechanical_links.rowid
ORDER BY
  mechanical_links.rowid
"#
        ))
    }
}

impl fmt::Display for LinkRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.source, self.relation, self.target)
    }
}

struct Held;

impl Anchors for Held {
    type Output = Option<Segment>;

    fn message(self, _message: &Uuid) -> Option<Segment> {
        None
    }

    fn segment(self, segment: &SegmentId) -> Option<Segment> {
        Some(Segment::new(segment.message().clone(), Ordinal::new(segment.ordinal().into_inner())))
    }
}

impl Hit {
    pub(crate) fn matching(term: &Term, scope: Scope, statements: &mut impl Execute) -> Result<Vec<Hit>, StoreFailure> {
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
    pub(crate) fn matching(term: &Term) -> Prepared<RankRow> {
        store_ports::query!(
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
        )
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

    pub(crate) fn passing(session: &SessionId, statements: &mut impl Execute) -> Result<Vec<Skipped>, StoreFailure> {
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

    pub(crate) fn all(statements: &mut impl Execute) -> Result<Vec<CursorRow>, StoreFailure> {
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

    fn reconciled(&self, stored: Option<&CursorRow>, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        match stored {
            None => statements.one(CursorRow::created(&CursorRowNew::from(self))).map(|_created| Written::new(1)),
            Some(found) => self.revising(found, statements),
        }
    }

    fn revising(&self, found: &CursorRow, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        Some(found)
            .filter(|stored| *stored != self)
            .map_or_else(|| Ok(Written::default()), |_changed| statements.written(self.updated()))
    }
}

impl Row for MessageRow {
    fn inserted(&self) -> Prepared<()> {
        let MessageRow { uuid, session, parent, at, author, body } = self;
        store_ports::query!(
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
        )
    }
}

impl Row for SegmentRow {
    fn inserted(&self) -> Prepared<()> {
        let SegmentRow { message, ordinal, shape, body } = self;
        store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO segments (message, ordinal, shape, body)
VALUES
  (?1, ?2, ?3, ?4)
"#,
            message: Uuid,
            ordinal: Ordinal,
            shape: Shape,
            body: Body
        )
    }
}

impl Row for LinkRow {
    fn inserted(&self) -> Prepared<()> {
        let LinkRow { source, relation, target, source_message, target_message } = self;
        store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO mechanical_links (
    source,
    relation,
    target,
    source_message,
    target_message
  )
VALUES
  (?1, ?2, ?3, ?4, ?5)
"#,
            source: Anchor,
            relation: Relation,
            target: Anchor,
            source_message: Uuid,
            target_message: Uuid
        )
    }
}

impl Row for EdgeRow {
    fn inserted(&self) -> Prepared<()> {
        let EdgeRow { session, uuid, parent } = self;
        store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO edges (session, uuid, parent)
VALUES
  (?1, ?2, ?3)
"#,
            session: SessionId,
            uuid: Uuid,
            parent: Option<Uuid>
        )
    }
}

impl CursorRow {
    pub(crate) fn written(&self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        let stored = statements.rows(CursorRow::fetched(&self.session))?;
        self.reconciled(stored.first(), statements)
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

impl AsRef<Shape> for SegmentRow {
    fn as_ref(&self) -> &Shape {
        &self.shape
    }
}

impl AsRef<Relation> for LinkRow {
    fn as_ref(&self) -> &Relation {
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
