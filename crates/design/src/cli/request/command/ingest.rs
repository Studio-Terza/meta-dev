mod count;

use std::fmt;

use design_store::{Cursors, SessionCursors, SessionRecords};
use newtype::Counter;
use store_ports::{Store, Written};
use stream::{Message, Relation, Shape, Stream};
use transcript_ports::{Authorship, Cursor, Kind, Session, SessionId, Transcripts, Unrecognised};

use crate::cli::failure::Failure;
use count::{Counts, Label};

#[derive(Debug)]
struct ReadTo {
    session: SessionId,
    cursor: Cursor,
}

#[derive(Debug)]
struct Ingested {
    end: ReadTo,
    stream: Stream,
    written: Written,
}

#[derive(Debug)]
pub struct Tally {
    ends: Vec<ReadTo>,
    authorship: Counts<Authorship>,
    unrecognised: Counts<Kind>,
    forms: Counts<Shape>,
    relations: Counts<Relation>,
    written: Written,
}

#[derive(Debug)]
pub struct Ingest<'ports, T: Transcripts, S: Store> {
    transcripts: &'ports T,
    store: &'ports S,
    cursors: Cursors,
}

impl<'ports, T: Transcripts, S: Store> Ingest<'ports, T, S> {
    pub fn read(transcripts: &'ports T, store: &'ports S) -> Result<Tally, Failure> {
        let lock = store.lock()?;
        let ingest = Ingest { transcripts, store, cursors: store.read(SessionCursors)? };
        let sessions = transcripts.sessions()?;
        let ingested = sessions.iter().map(|session| ingest.session(session)).collect::<Result<Vec<_>, Failure>>()?;
        lock.end()?;
        Ok(Tally::counted(ingested))
    }

    fn session(&self, session: &Session) -> Result<Ingested, Failure> {
        let read = self.transcripts.read_from(session, self.cursors.resuming(session.stem()))?;
        let end = ReadTo::ending(session, read.cursor());
        let stream: Stream = read.into_iter().collect();
        let written = self.store.write(SessionRecords::read(&end.session, end.cursor, &stream))?;
        Ok(Ingested { end, stream, written })
    }
}

impl ReadTo {
    fn ending(session: &Session, cursor: Cursor) -> ReadTo {
        ReadTo { session: session.stem().clone(), cursor }
    }
}

impl Tally {
    fn counted(ingested: Vec<Ingested>) -> Tally {
        let written = Written::total(ingested.iter().map(|session| session.written));
        let (ends, streams): (Vec<ReadTo>, Vec<Stream>) =
            ingested.into_iter().map(|session| (session.end, session.stream)).unzip();
        let messages = || streams.iter().flat_map(|stream| stream.as_ref().iter());
        Tally {
            authorship: messages().map(|message| message.record().author()).collect(),
            unrecognised: streams.iter().flat_map(Stream::unrecognised).map(Unrecognised::kind).cloned().collect(),
            forms: messages().flat_map(Message::segments).map(|segment| Shape::from(segment.form())).collect(),
            relations: streams.iter().flat_map(Stream::links).map(|link| link.relation()).collect(),
            written,
            ends,
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ends.iter().try_for_each(|end| writeln!(formatter, "session {} read to {}", end.session, end.cursor))?;
        self.authorship.lines(Label::AUTHORSHIP, formatter)?;
        self.unrecognised.lines(Label::UNRECOGNISED, formatter)?;
        self.forms.lines(Label::FORM, formatter)?;
        self.relations.lines(Label::RELATION, formatter)?;
        writeln!(formatter, "{} {}", Label::WRITTEN, self.written.count())
    }
}
