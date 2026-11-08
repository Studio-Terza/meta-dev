mod count;

use design_store::{Cursors, EarlierLines, SessionCursors, SessionRecords};
use newtype::Counter;
use store_ports::{Store, Written};
use stream::{Message, Relation, Shape, Stream};
use transcript_ports::{Authorship, Cursor, Kind, Line, Session, SessionId, Transcripts, Unrecognised};

use crate::cli::failure::Failure;
use crate::cli::output::{self, Count};
use count::Counts;

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
            .collect::<Result<Vec<_>, Failure>>()?;
        lock.end()?;
        Ok(Tally::counted(ingested))
    }

    fn session(&self, session: &Session, cursor: Cursor) -> Result<Ingested, Failure> {
        let read = self.transcripts.read_from(session, cursor)?;
        let end = ReadTo::ending(session, read.cursor());
        let lines: Vec<Line> = read.into_iter().collect();
        let earlier = lines.first().map(|_| self.store.read(EarlierLines::within(end.session.clone()))).transpose()?;
        let stream = earlier.unwrap_or_default().resumed(lines);
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

impl Tally {
    pub fn lines(&self) -> Vec<output::Line> {
        self.ends
            .iter()
            .map(|end| output::Line::Read { session: end.session.clone(), cursor: end.cursor })
            .chain(self.authorship.lines(|author, count| output::Line::Authored { author, count }))
            .chain(self.unrecognised.lines(|kind, count| output::Line::Unrecognised { kind, count }))
            .chain(self.forms.lines(|shape, count| output::Line::Formed { shape, count }))
            .chain(self.relations.lines(|relation, count| output::Line::Related { relation, count }))
            .chain([output::Line::Written(Count::new(self.written.count()))])
            .collect()
    }
}
