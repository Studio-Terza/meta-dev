pub(crate) mod ignored;

use file_ports::Offset;
use newtype::Wrapping;
use store_ports::{Columns, Execute, Prepared, StoreFailure};
use transcript_ports::{Authorship, Body, Cursor, Instant, Record, SessionId, Uuid};

#[derive(Debug)]
pub struct MessageRow {
    uuid: Uuid,
    session: SessionId,
    parent: Option<Uuid>,
    at: Instant,
    author: Authorship,
    body: Body,
}

#[derive(Clone, Debug, PartialEq, Eq, Columns)]
pub struct CursorRow {
    session: SessionId,
    read_to: Offset,
}

impl From<&Record> for MessageRow {
    fn from(record: &Record) -> MessageRow {
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

impl MessageRow {
    pub(crate) fn inserted(&self) -> Prepared<()> {
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

    pub(crate) fn upserted(&self) -> Prepared<()> {
        let CursorRow { session, read_to } = self;
        store_ports::query!(
            r#"
INSERT INTO
  cursors (session, read_to)
VALUES
  (?1, ?2)
ON CONFLICT (session) DO UPDATE
SET
  read_to = excluded.read_to
WHERE
  read_to != excluded.read_to
"#,
            session: SessionId,
            read_to: Offset
        )
    }
}

impl From<&CursorRow> for Cursor {
    fn from(row: &CursorRow) -> Cursor {
        Cursor::new(row.read_to)
    }
}

impl AsRef<Authorship> for MessageRow {
    fn as_ref(&self) -> &Authorship {
        &self.author
    }
}
