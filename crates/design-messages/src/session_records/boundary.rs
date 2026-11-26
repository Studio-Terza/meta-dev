use std::collections::BTreeSet;

use design_model::Segment;
use store_ports::{Batch, Counts, Done, Execute, Query, StoreFailure};
use stream::Message;
use transcript_ports::{Record, Uuid};

use super::{Anchoring, Held, Holding, SessionRecords};
use crate::boundary::{LinkRow, MessageRow, Row, SegmentRow};

impl SessionRecords {
    fn all<D: Done>(rows: &[impl Row], statements: &mut impl Execute) -> Result<D, StoreFailure> {
        statements.counted_all(rows.iter().map(Row::inserted))
    }
}

impl Anchoring {
    fn read(records: &SessionRecords, statements: &mut impl Execute) -> Result<Anchoring, StoreFailure> {
        let named: BTreeSet<Segment> = records.links.iter().flat_map(LinkRow::segments).collect();
        let speakers: BTreeSet<&Uuid> = named.iter().map(AsRef::<Uuid>::as_ref).collect();
        let stored = statements.read_all(speakers.iter().map(|speaker| SegmentRow::stored_for(speaker)))?;
        let said = statements.read_all(speakers.iter().map(|speaker| MessageRow::named(speaker)))?;
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

    fn rows(self, statements: &mut impl Execute) -> Result<Held, StoreFailure> {
        let Holding(records) = self;
        let anchoring = Anchoring::read(&records, statements)?;
        let (links, refused): (Vec<LinkRow>, Vec<LinkRow>) =
            records.links.iter().cloned().partition(|link| anchoring.unheld(link).is_none());
        Ok(Held(SessionRecords { links, refused, ..records }))
    }
}

impl Batch for Held {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        self.counted(statements)
    }
}

impl Counts for Held {
    fn counted<D: Done>(self, statements: &mut impl Execute) -> Result<D, StoreFailure> {
        let Held(records) = self;
        let written = [
            SessionRecords::all(&records.messages, statements)?,
            SessionRecords::all(&records.segments, statements)?,
            SessionRecords::all(&records.links, statements)?,
            SessionRecords::all(&records.edges, statements)?,
            records.cursor.written(statements)?,
        ];
        Ok(D::total(written.into_iter()))
    }
}
