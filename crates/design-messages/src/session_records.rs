use port_core::Op;
use store_ports::{Batch, Execute, StoreFailure, Written};
use transcript_ports::{Authorship, Cursor, Kind, Line, SessionId};

use crate::boundary::{CursorRow, MessageRow};
use crate::ended::EndedRow;

#[derive(Debug, Op)]
pub struct SessionRecords {
    messages: Vec<MessageRow>,
    ended: Vec<EndedRow>,
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
                Line::Ended(_) | Line::Unrecognised(_) | Line::Skipped(_) => None,
            })
            .collect();
        let ended = lines
            .iter()
            .filter_map(|line| match line {
                Line::Ended(task) => Some(EndedRow::new(task.clone())),
                Line::Said(_) | Line::Unrecognised(_) | Line::Skipped(_) => None,
            })
            .collect();
        let unrecognised = lines
            .iter()
            .filter_map(|line| match line {
                Line::Unrecognised(unread) => Some(unread.kind().clone()),
                Line::Said(_) | Line::Ended(_) | Line::Skipped(_) => None,
            })
            .collect();
        SessionRecords { messages, ended, unrecognised, cursor: CursorRow::reached(session, cursor) }
    }

    pub fn authors(&self) -> impl Iterator<Item = Authorship> + '_ {
        self.messages.iter().map(|message| *AsRef::<Authorship>::as_ref(message))
    }

    #[must_use]
    pub fn unrecognised(&self) -> &[Kind] {
        &self.unrecognised
    }
}

impl Batch for SessionRecords {
    type Failure = StoreFailure;

    type Done = Written;

    fn written(self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        let messages = self.messages.iter().map(MessageRow::inserted);
        let ended = self.ended.iter().map(EndedRow::inserted);
        statements.written(messages.chain(ended).chain([self.cursor.upserted()]))
    }
}
