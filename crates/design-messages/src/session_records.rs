mod boundary;

use std::collections::BTreeMap;

use design_model::Segment;
use port_core::Op;
use stream::Stream;
use transcript_ports::{Cursor, SessionId};

use crate::boundary::{CursorRow, EdgeRow, LinkRow, MessageRow, SegmentRow};

#[derive(Debug, Op)]
pub struct SessionRecords {
    messages: Vec<MessageRow>,
    segments: Vec<SegmentRow>,
    links: Vec<LinkRow>,
    refused: Vec<LinkRow>,
    edges: Vec<EdgeRow>,
    cursor: CursorRow,
}

#[derive(Debug, Op)]
pub struct Holding(SessionRecords);

#[derive(Debug, Op)]
pub struct Held(SessionRecords);

struct Anchoring {
    landing: BTreeMap<Segment, SegmentRow>,
    deriving: BTreeMap<Segment, SegmentRow>,
}

impl SessionRecords {
    #[must_use]
    pub fn read(session: &SessionId, cursor: Cursor, stream: &Stream) -> SessionRecords {
        let messages = stream.as_ref();
        SessionRecords {
            messages: messages.iter().map(MessageRow::from).collect(),
            segments: messages.iter().flat_map(SegmentRow::within).collect(),
            links: stream.links().iter().map(LinkRow::from).collect(),
            refused: Vec::new(),
            edges: messages
                .iter()
                .map(|message| EdgeRow::said(session, message))
                .chain(stream.skipped().iter().map(|skipped| EdgeRow::skipped(session, skipped)))
                .collect(),
            cursor: CursorRow::reached(session, cursor),
        }
    }
}

impl Anchoring {
    fn keyed(rows: impl Iterator<Item = SegmentRow>) -> BTreeMap<Segment, SegmentRow> {
        rows.map(|row| (row.segment(), row)).collect()
    }

    fn unheld(&self, link: &LinkRow) -> Option<Segment> {
        link.segments()
            .into_iter()
            .find(|segment| self.landing.get(segment).is_none_or(|row| self.deriving.get(segment) != Some(row)))
    }
}

impl From<SessionRecords> for Holding {
    fn from(records: SessionRecords) -> Holding {
        Holding(records)
    }
}

impl Held {
    #[must_use]
    pub fn refused(&self) -> &[LinkRow] {
        &self.0.refused
    }
}
