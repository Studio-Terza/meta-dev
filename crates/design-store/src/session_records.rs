use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, Statements, StoreFailure, Written};
use stream::Stream;
use transcript_ports::{Cursor, SessionId, Unbare};

use crate::boundary::{CursorRow, EdgeRow, LinkRow, MessageRow, Row, SegmentRow};

#[derive(Debug, Op)]
pub struct SessionRecords {
    messages: Vec<MessageRow>,
    segments: Vec<SegmentRow>,
    links: Vec<LinkRow>,
    edges: Vec<EdgeRow>,
    cursor: CursorRow,
}

impl SessionRecords {
    pub fn read(session: &SessionId, cursor: Cursor, stream: &Stream) -> Result<SessionRecords, Unbare> {
        let messages = stream.as_ref();
        Ok(SessionRecords {
            messages: messages.iter().map(MessageRow::from).collect(),
            segments: messages.iter().flat_map(SegmentRow::within).collect(),
            links: stream.links()?.iter().map(LinkRow::from).collect(),
            edges: messages
                .iter()
                .map(|message| EdgeRow::said(session, message))
                .chain(stream.skipped().iter().map(|skipped| EdgeRow::skipped(session, skipped)))
                .collect(),
            cursor: CursorRow::reached(session, cursor),
        })
    }

    fn all(rows: &[impl Row], statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let written = rows.iter().map(|row| row.written(statements)).collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

impl Batch for SessionRecords {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let written = [
            Self::all(&self.messages, statements)?,
            Self::all(&self.segments, statements)?,
            Self::all(&self.links, statements)?,
            Self::all(&self.edges, statements)?,
            self.cursor.written(statements)?,
        ];
        Ok(Written::total(written.into_iter()))
    }
}
