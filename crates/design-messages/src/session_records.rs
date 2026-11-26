use port_core::Op;
use store_ports::{Batch, Counts, Done, Execute, StoreFailure};
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
    pub fn read(session: &SessionId, cursor: Cursor, lines: Vec<Line>) -> SessionRecords {
        let (messages, unrecognised) = lines.into_iter().fold((Vec::new(), Vec::new()), |(mut said, mut odd), line| {
            match line {
                Line::Said(record) => said.push(MessageRow::from(&record)),
                Line::Unrecognised(unread) => odd.push(unread.kind().clone()),
                Line::Skipped(_) => {}
            }
            (said, odd)
        });
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

impl Batch for SessionRecords {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        self.counted(statements)
    }
}

impl Counts for SessionRecords {
    fn counted<D: Done>(self, statements: &mut impl Execute) -> Result<D, StoreFailure> {
        let written: [D; 2] = [
            statements.counted_all(self.messages.iter().map(MessageRow::inserted))?,
            self.cursor.written(statements)?,
        ];
        Ok(D::total(written.into_iter()))
    }
}
