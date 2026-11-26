pub(crate) mod ignored;

use std::collections::BTreeMap;

use file_ports::Offset;
use newtype::Wrapping;
use store_ports::Record as _;
use store_ports::{Columns, Done, Execute, Prepared, StoreFailure};
use transcript_ports::{Authorship, Body, Cursor, Instant, Record, SessionId, Uuid};

use crate::Said;

#[derive(Debug)]
pub struct MessageRow {
    uuid: Uuid,
    session: SessionId,
    parent: Option<Uuid>,
    at: Instant,
    author: Authorship,
    body: Body,
}

#[derive(Debug, Columns)]
pub(crate) struct SaidRow {
    uuid: Uuid,
    at: Instant,
    author: Authorship,
    body: Body,
}

#[derive(Clone, Debug, PartialEq, Eq, Columns, store_ports::Record)]
#[record(table = "cursors")]
pub struct CursorRow {
    #[record(key)]
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

impl From<SaidRow> for Said {
    fn from(row: SaidRow) -> Said {
        Said::new(row.uuid, row.at, row.body)
    }
}

impl SaidRow {
    pub(crate) fn last_in(
        session: &SessionId,
        author: Authorship,
        statements: &mut impl Execute,
    ) -> Result<Vec<SaidRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            SaidRow,
            r#"
SELECT
  uuid,
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
        let owned: BTreeMap<(Instant, Uuid), SaidRow> = SaidRow::all(statements)?
            .into_iter()
            .filter(|row| row.author == Authorship::Owner)
            .map(|row| ((row.at.clone(), row.uuid.clone()), row))
            .collect();
        Ok(owned.into_values().map(Said::from).collect())
    }

    fn all(statements: &mut impl Execute) -> Result<Vec<SaidRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            SaidRow,
            r#"
SELECT
  uuid,
  at,
  author,
  body
FROM
  messages
"#
        ))
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

    pub(crate) fn written<D: Done>(&self, statements: &mut impl Execute) -> Result<D, StoreFailure> {
        let stored = statements.rows(CursorRow::fetched(&self.session))?;
        self.reconciled(stored.first(), statements)
    }

    fn reconciled<D: Done>(
        &self,
        stored: Option<&CursorRow>,
        statements: &mut impl Execute,
    ) -> Result<D, StoreFailure> {
        match stored {
            None => statements.one(CursorRow::created(&CursorRowNew::from(self))).map(|_created| D::changed(1)),
            Some(found) => self.revising(found, statements),
        }
    }

    fn revising<D: Done>(&self, found: &CursorRow, statements: &mut impl Execute) -> Result<D, StoreFailure> {
        Some(found)
            .filter(|stored| *stored != self)
            .map_or_else(|| Ok(D::unchanged()), |_changed| statements.done(self.updated()))
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
