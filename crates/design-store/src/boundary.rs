use file_ports::Offset;
use newtype::{Arbitrary, Display, Newtype};
use store_ports::{Statements, StoreFailure, Written};
use stream::{MechanicalLink, Message, Shape};
use transcript_ports::{Body, Cursor, Instant, SessionId, Uuid};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Author(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct ShapeName(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct RelationName(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct AnchorName(String);

pub(crate) trait Row {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure>;
}

#[derive(Debug)]
pub struct MessageRow {
    uuid: String,
    session: String,
    parent: Option<String>,
    at: String,
    author: String,
    body: String,
}

#[derive(Debug)]
pub struct SegmentRow {
    message: String,
    ordinal: i64,
    shape: String,
    body: String,
}

#[derive(Debug)]
pub struct LinkRow {
    source: String,
    relation: String,
    target: String,
    source_message: String,
    target_message: String,
}

#[derive(Debug)]
pub(crate) struct CursorRow {
    session: String,
    read_to: i64,
}

impl From<&Message> for MessageRow {
    fn from(message: &Message) -> MessageRow {
        let record = message.record();
        MessageRow {
            uuid: record.uuid().to_string(),
            session: record.session().to_string(),
            parent: record.parent().map(ToString::to_string),
            at: record.stamp().to_string(),
            author: record.author().to_string(),
            body: record.text().to_string(),
        }
    }
}

impl From<&SegmentRow> for Body {
    fn from(segment: &SegmentRow) -> Body {
        Body::new(segment.body.clone())
    }
}

impl From<&MechanicalLink> for LinkRow {
    fn from(link: &MechanicalLink) -> LinkRow {
        let [source, target] = link.anchors();
        LinkRow {
            source: source.to_string(),
            relation: link.relation().to_string(),
            target: target.to_string(),
            source_message: source.message().to_string(),
            target_message: target.message().to_string(),
        }
    }
}

impl MessageRow {
    pub(crate) fn named(uuid: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<MessageRow>, StoreFailure> {
        let uuid = uuid.to_string();
        statements.rows(sqlx::query_as!(
            MessageRow,
            "SELECT uuid, session, parent, at, author, body FROM messages WHERE uuid = ?1",
            uuid
        ))
    }

    #[must_use]
    pub fn uuid(&self) -> Uuid {
        Uuid::new(self.uuid.clone())
    }

    #[must_use]
    pub fn session(&self) -> SessionId {
        SessionId::new(self.session.clone())
    }

    #[must_use]
    pub fn stamp(&self) -> Instant {
        Instant::new(self.at.clone())
    }

    #[must_use]
    pub fn author(&self) -> Author {
        Author::new(self.author.clone())
    }
}

impl SegmentRow {
    pub(crate) fn within(message: &Message) -> Vec<SegmentRow> {
        let uuid = message.record().uuid().to_string();
        message
            .segments()
            .iter()
            .map(|segment| SegmentRow {
                message: uuid.clone(),
                ordinal: i64::try_from(segment.identity().ordinal().into_inner()).unwrap_or(i64::MAX),
                shape: Shape::from(segment.form()).to_string(),
                body: segment.text(message).to_string(),
            })
            .collect()
    }

    pub(crate) fn stored_for(uuid: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<SegmentRow>, StoreFailure> {
        let uuid = uuid.to_string();
        statements.rows(sqlx::query_as!(
            SegmentRow,
            "SELECT message, ordinal, shape, body FROM segments WHERE message = ?1 ORDER BY ordinal",
            uuid
        ))
    }

    #[must_use]
    pub fn ordinal(&self) -> i64 {
        self.ordinal
    }

    #[must_use]
    pub fn shape(&self) -> ShapeName {
        ShapeName::new(self.shape.clone())
    }
}

impl LinkRow {
    pub(crate) fn touching(uuid: &Uuid, statements: &mut Statements<'_>) -> Result<Vec<LinkRow>, StoreFailure> {
        let uuid = uuid.to_string();
        statements.rows(sqlx::query_as!(
            LinkRow,
            "SELECT source, relation, target, source_message, target_message FROM mechanical_links WHERE ?1 IN (source_message, target_message) ORDER BY rowid",
            uuid
        ))
    }

    #[must_use]
    pub fn anchors(&self) -> [AnchorName; 2] {
        [AnchorName::new(self.source.clone()), AnchorName::new(self.target.clone())]
    }

    #[must_use]
    pub fn relation(&self) -> RelationName {
        RelationName::new(self.relation.clone())
    }
}

impl CursorRow {
    pub(crate) fn reached(session: &SessionId, cursor: Cursor) -> CursorRow {
        CursorRow {
            session: session.to_string(),
            read_to: i64::try_from(cursor.into_inner().into_inner()).unwrap_or(i64::MAX),
        }
    }

    pub(crate) fn all(statements: &mut Statements<'_>) -> Result<Vec<CursorRow>, StoreFailure> {
        statements.rows(sqlx::query_as!(CursorRow, "SELECT session, read_to FROM cursors"))
    }

    pub(crate) fn session(&self) -> SessionId {
        SessionId::new(self.session.clone())
    }

    pub(crate) fn cursor(&self) -> Cursor {
        Cursor::new(Offset::new(u64::try_from(self.read_to).unwrap_or_default()))
    }
}

impl Row for MessageRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO messages (uuid, session, parent, at, author, body) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            self.uuid,
            self.session,
            self.parent,
            self.at,
            self.author,
            self.body
        ))
    }
}

impl Row for SegmentRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO segments (message, ordinal, shape, body) VALUES (?1, ?2, ?3, ?4)",
            self.message,
            self.ordinal,
            self.shape,
            self.body
        ))
    }
}

impl Row for LinkRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO mechanical_links (source, relation, target, source_message, target_message) VALUES (?1, ?2, ?3, ?4, ?5)",
            self.source,
            self.relation,
            self.target,
            self.source_message,
            self.target_message
        ))
    }
}

impl Row for CursorRow {
    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!(
            "INSERT INTO cursors (session, read_to) VALUES (?1, ?2) ON CONFLICT (session) DO UPDATE SET read_to = excluded.read_to WHERE read_to <> excluded.read_to",
            self.session,
            self.read_to
        ))
    }
}
