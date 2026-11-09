use std::collections::{BTreeMap, BTreeSet};

use design_model::Segment;
use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, Query, Statements, StoreFailure, Written};
use stream::{Message, Stream};
use transcript_ports::{Cursor, Record, SessionId, Uuid};

use crate::boundary::{CursorRow, EdgeRow, LinkRow, MessageRow, Row, SegmentRow};

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

    #[must_use]
    pub fn refused(&self) -> &[LinkRow] {
        &self.refused
    }

    fn all(rows: &[impl Row], statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let written = rows.iter().map(|row| row.written(statements)).collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

impl Anchoring {
    fn read(records: &SessionRecords, statements: &mut Statements<'_>) -> Result<Anchoring, StoreFailure> {
        let named: BTreeSet<Segment> = records.links.iter().flat_map(LinkRow::segments).collect();
        let speakers: BTreeSet<&Uuid> = named.iter().map(AsRef::<Uuid>::as_ref).collect();
        let stored = speakers
            .iter()
            .map(|speaker| SegmentRow::stored_for(speaker, statements))
            .collect::<Result<Vec<Vec<SegmentRow>>, StoreFailure>>()?;
        let said = speakers
            .iter()
            .map(|speaker| MessageRow::named(speaker, statements))
            .collect::<Result<Vec<Vec<MessageRow>>, StoreFailure>>()?;
        let derived = said.into_iter().flatten().map(Record::from).map(Message::from);
        let fresh = records.segments.iter().cloned();
        Ok(Anchoring {
            landing: Anchoring::keyed(fresh.clone().chain(stored.into_iter().flatten())),
            deriving: Anchoring::keyed(derived.flat_map(|message| SegmentRow::within(&message)).chain(fresh)),
        })
    }

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

impl Query for Holding {
    type Rows = SessionRecords;

    fn rows(self, statements: &mut Statements<'_>) -> Result<SessionRecords, StoreFailure> {
        let Holding(records) = self;
        let anchoring = Anchoring::read(&records, statements)?;
        let (links, refused): (Vec<LinkRow>, Vec<LinkRow>) =
            records.links.iter().cloned().partition(|link| anchoring.unheld(link).is_none());
        Ok(SessionRecords { links, refused, ..records })
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
