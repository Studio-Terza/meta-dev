mod boundary;

use std::fmt;

use design_messages::{Cursors, SessionCursors, SessionRecords};
use newtype::Counter;
use store_ports::{Counted, Store, Written};
use transcript_ports::{Authorship, Cursor, Kind, Line, Session, SessionId, Transcripts};

use crate::cli::Count;
use crate::cli::failure::Failure;
use boundary::Counts;

#[derive(Debug)]
struct ReadTo {
    session: SessionId,
    cursor: Cursor,
}

#[derive(Debug)]
struct Ingested {
    end: ReadTo,
    authors: Vec<Authorship>,
    unrecognised: Vec<Kind>,
    written: Written,
}

#[derive(Debug)]
pub(crate) struct Tally {
    ends: Vec<ReadTo>,
    authorship: Counts<Authorship>,
    unrecognised: Counts<Kind>,
    written: Written,
}

#[derive(Debug)]
pub(crate) struct Ingest<'ports, T: Transcripts, S: Store> {
    transcripts: &'ports T,
    store: &'ports S,
    cursors: Cursors,
}

impl<'ports, T: Transcripts, S: Store> Ingest<'ports, T, S> {
    pub(crate) fn read(store: &'ports S, transcripts: &'ports T) -> Result<Tally, Failure> {
        let lock = store.lock()?;
        let ingest = Ingest { transcripts, store, cursors: store.read(SessionCursors)? };
        let sessions = transcripts.sessions()?;
        let ingested = sessions
            .iter()
            .filter_map(|session| {
                session.unread(ingest.cursors.resuming(session.stem())).map(|cursor| (session, cursor))
            })
            .map(|(session, cursor)| ingest.session(session, cursor))
            .collect::<Result<Vec<_>, Failure>>()?;
        drop(lock);
        Ok(Tally::counted(ingested))
    }

    fn session(&self, session: &Session, cursor: Cursor) -> Result<Ingested, Failure> {
        let read = self.transcripts.read_from(session, cursor)?;
        let end = ReadTo { session: session.stem().clone(), cursor: read.cursor() };
        let lines: Vec<Line> = read.into_iter().collect();
        let records = SessionRecords::read(&end.session, end.cursor, &lines);
        let authors = records.authors().collect();
        let unrecognised = records.unrecognised().to_vec();
        let written = self.store.write(Counted::new(records))?;
        Ok(Ingested { end, authors, unrecognised, written })
    }
}

impl Tally {
    fn counted(ingested: Vec<Ingested>) -> Tally {
        let written = <Written as Counter>::total(ingested.iter().map(|session| session.written));
        let authorship = ingested.iter().flat_map(|session| session.authors.iter().copied()).collect();
        let unrecognised = ingested.iter().flat_map(|session| session.unrecognised.iter().cloned()).collect();
        Tally { ends: ingested.into_iter().map(|session| session.end).collect(), authorship, unrecognised, written }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ends.iter().try_for_each(|end| writeln!(formatter, "session {} read to {}", end.session, end.cursor))?;
        self.authorship.listed(formatter, "authorship")?;
        self.unrecognised.listed(formatter, "unrecognised")?;
        writeln!(formatter, "written {}", Count::new(self.written.count()))
    }
}
