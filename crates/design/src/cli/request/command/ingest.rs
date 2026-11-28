use std::collections::BTreeMap;
use std::fmt;

use design_messages::{Cursors, SessionCursors, SessionRecords};
use newtype::Wrapping;
use pipeline::Measure;
use store_ports::{Store, Written};
use transcript_ports::{Authorship, Cursor, Kind, Line, Session, SessionId, Transcripts};

use crate::cli::Count;
use crate::cli::failure::Failure;

#[derive(Debug)]
struct ReadTo {
    session: SessionId,
    cursor: Cursor,
}

#[derive(Debug, Measure)]
pub struct Tally {
    ends: Vec<ReadTo>,
    authorship: BTreeMap<Authorship, Count>,
    unrecognised: BTreeMap<Kind, Count>,
    written: Written,
}

#[derive(Debug)]
pub struct Ingest<'ports, T: Transcripts, S: Store> {
    transcripts: &'ports T,
    store: &'ports S,
    cursors: Cursors,
}

impl<'ports, T: Transcripts, S: Store> Ingest<'ports, T, S> {
    pub fn read(store: &'ports S, transcripts: &'ports T) -> Result<Tally, Failure> {
        let lock = store.lock()?;
        let ingest = Ingest { transcripts, store, cursors: store.read(SessionCursors)? };
        let sessions = transcripts.sessions()?;
        let ingested = sessions
            .iter()
            .filter_map(|session| {
                session.unread(ingest.cursors.resuming(session.stem())).map(|cursor| (session, cursor))
            })
            .map(|(session, cursor)| ingest.session(session, cursor))
            .collect::<Result<Vec<Tally>, Failure>>()?;
        lock.end()?;
        Ok(Tally::total(ingested))
    }

    fn session(&self, session: &Session, cursor: Cursor) -> Result<Tally, Failure> {
        let read = self.transcripts.read_from(session, cursor)?;
        let end = ReadTo { session: session.stem().clone(), cursor: read.cursor() };
        let lines: Vec<Line> = read.into_iter().collect();
        let records = SessionRecords::read(&end.session, end.cursor, &lines);
        let authorship = BTreeMap::total(records.authors().map(|author| BTreeMap::from([(author, Count::new(1))])));
        let unrecognised =
            BTreeMap::total(records.unrecognised().iter().map(|kind| BTreeMap::from([(kind.clone(), Count::new(1))])));
        Ok(Tally { ends: vec![end], authorship, unrecognised, written: self.store.write(records)? })
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ends.iter().try_for_each(|end| writeln!(formatter, "session {} read to {}", end.session, end.cursor))?;
        self.authorship.iter().try_for_each(|(author, count)| writeln!(formatter, "authorship {author} {count}"))?;
        self.unrecognised.iter().try_for_each(|(kind, count)| writeln!(formatter, "unrecognised {kind} {count}"))?;
        writeln!(formatter, "written {}", Count::new(self.written.into_inner()))
    }
}
