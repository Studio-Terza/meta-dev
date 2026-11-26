use std::collections::BTreeSet;

use design_model::Segment;
use newtype::Counter;
use store_ports::{Batch, Query, Statements, StoreFailure, Written};
use stream::Message;
use transcript_ports::{Record, Uuid};

use super::{Anchoring, Held, Holding, SessionRecords};
use crate::boundary::{LinkRow, MessageRow, Row, SegmentRow};

impl SessionRecords {
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
}

impl Query for Holding {
    type Rows = Held;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Held, StoreFailure> {
        let Holding(records) = self;
        let anchoring = Anchoring::read(&records, statements)?;
        let (links, refused): (Vec<LinkRow>, Vec<LinkRow>) =
            records.links.iter().cloned().partition(|link| anchoring.unheld(link).is_none());
        Ok(Held(SessionRecords { links, refused, ..records }))
    }
}

impl Batch for Held {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let Held(records) = self;
        let written = [
            SessionRecords::all(&records.messages, statements)?,
            SessionRecords::all(&records.events, statements)?,
            SessionRecords::all(&records.segments, statements)?,
            SessionRecords::all(&records.links, statements)?,
            SessionRecords::all(&records.edges, statements)?,
            records.cursor.written(statements)?,
        ];
        Ok(Written::total(written.into_iter()))
    }
}
