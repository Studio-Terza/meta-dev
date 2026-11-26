use port_core::Op;
use store_ports::{Counts, Done, Execute, StoreFailure};
use transcript_ports::{Authorship, Cursor, Kind, Line, SessionId};

use crate::boundary::{CursorRow, MessageRow};

#[derive(Debug, Op)]
pub struct SessionRecords {
    messages: Vec<MessageRow>,
    unrecognised: Vec<Kind>,
    cursor: CursorRow,
}

impl SessionRecords {
    #[must_use]
    pub fn read(session: &SessionId, cursor: Cursor, lines: &[Line]) -> SessionRecords {
        let messages = lines
            .iter()
            .filter_map(|line| match line {
                Line::Said(record) => Some(MessageRow::from(record)),
                Line::Unrecognised(_) | Line::Skipped(_) => None,
            })
            .collect();
        let unrecognised = lines
            .iter()
            .filter_map(|line| match line {
                Line::Unrecognised(unread) => Some(unread.kind().clone()),
                Line::Said(_) | Line::Skipped(_) => None,
            })
            .collect();
        SessionRecords { messages, unrecognised, cursor: CursorRow::reached(session, cursor) }
    }

    pub fn authors(&self) -> impl Iterator<Item = Authorship> + '_ {
        self.messages.iter().map(|message| *AsRef::<Authorship>::as_ref(message))
    }

    #[must_use]
    pub fn unrecognised(&self) -> &[Kind] {
        &self.unrecognised
    }
}

impl Counts for SessionRecords {
    type Failure = StoreFailure;

    fn counted<D: Done>(self, statements: &mut impl Execute) -> Result<D, StoreFailure> {
        let messages = statements.counted_all(self.messages.iter().map(MessageRow::inserted))?;
        Ok(D::total([messages, self.cursor.written(statements)?].into_iter()))
    }
}
