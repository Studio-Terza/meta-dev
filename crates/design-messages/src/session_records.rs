use pipeline::Measure;
use port_core::Op;
use store_ports::{Batch, Execute, StoreFailure, Written};
use transcript_ports::{Authorship, Cursor, Kind, Line, SessionId};

use crate::boundary::{CursorRow, MessageRow};
use crate::ended::EndedRow;

#[derive(Debug, Op)]
pub struct SessionRecords {
    lifted: Lifted,
    cursor: CursorRow,
}

pipeline::measured! {
    #[derive(Debug)]
    struct Lifted {
        messages: Vec<MessageRow>,
        ended: Vec<EndedRow>,
        unrecognised: Vec<Kind>,
    }
}

impl SessionRecords {
    #[must_use]
    pub fn read(session: &SessionId, cursor: Cursor, lines: &[Line]) -> SessionRecords {
        SessionRecords {
            lifted: Lifted::total(lines.iter().map(Lifted::from)),
            cursor: CursorRow::reached(session, cursor),
        }
    }

    pub fn authors(&self) -> impl Iterator<Item = Authorship> + '_ {
        self.lifted.messages.iter().map(|message| *AsRef::<Authorship>::as_ref(message))
    }

    #[must_use]
    pub fn unrecognised(&self) -> &[Kind] {
        &self.lifted.unrecognised
    }
}

impl From<&Line> for Lifted {
    fn from(line: &Line) -> Lifted {
        match line {
            Line::Said(record) => Lifted { messages: vec![MessageRow::from(record)], ..Lifted::default() },
            Line::Ended(task) => Lifted { ended: vec![EndedRow::new(task.clone())], ..Lifted::default() },
            Line::Unrecognised(unread) => Lifted { unrecognised: vec![unread.kind().clone()], ..Lifted::default() },
            Line::Skipped(_) => Lifted::default(),
        }
    }
}

impl Batch for SessionRecords {
    type Failure = StoreFailure;

    type Done = Written;

    fn written(self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        let messages = self.lifted.messages.iter().map(MessageRow::inserted);
        let ended = self.lifted.ended.iter().map(EndedRow::inserted);
        statements.written(messages.chain(ended).chain([self.cursor.upserted()]))
    }
}
