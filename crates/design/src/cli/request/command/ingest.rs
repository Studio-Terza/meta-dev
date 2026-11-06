mod count;

use std::fmt;

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
pub struct Tally {
    ends: Vec<ReadTo>,
    authorship: Counts<Authorship>,
    unrecognised: Counts<Kind>,
    forms: Counts<Shape>,
    relations: Counts<Relation>,
}

#[derive(Debug)]
pub struct Ingest;

impl Ingest {
    pub fn read(port: &impl Transcripts) -> Result<Tally, Failure> {
        let sessions = port.sessions()?;
        let reads = port.read_all(&sessions)?;
        let ends = sessions.iter().zip(&reads).map(|(session, read)| ReadTo::ending(session, read.cursor())).collect();
        let streams: Vec<Stream> = reads.into_iter().map(|read| read.into_iter().collect()).collect();
        Ok(Tally::counted(ends, &streams))
    }
}

impl ReadTo {
    fn ending(session: &Session, cursor: Cursor) -> ReadTo {
        ReadTo { session: session.stem().clone(), cursor }
    }
}

impl Tally {
    fn counted(ends: Vec<ReadTo>, streams: &[Stream]) -> Tally {
        let messages = || streams.iter().flat_map(|stream| stream.as_ref().iter());
        Tally {
            ends,
            authorship: messages().map(|message| message.record().author()).collect(),
            unrecognised: streams.iter().flat_map(Stream::unrecognised).map(Unrecognised::kind).cloned().collect(),
            forms: messages().flat_map(Message::segments).map(|segment| Shape::from(segment.form())).collect(),
            relations: streams.iter().flat_map(Stream::links).map(|link| link.relation()).collect(),
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ends.iter().try_for_each(|end| writeln!(formatter, "session {} read to {}", end.session, end.cursor))?;
        self.authorship.lines(Label::AUTHORSHIP, formatter)?;
        self.unrecognised.lines(Label::UNRECOGNISED, formatter)?;
        self.forms.lines(Label::FORM, formatter)?;
        self.relations.lines(Label::RELATION, formatter)
    }
}
