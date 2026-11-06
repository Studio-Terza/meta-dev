mod segmenter;

use grammar::message::Lexemes;
use transcript_ports::Record;

use crate::segment::Segment;
use segmenter::Segmenter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message(Record);

impl From<Record> for Message {
    fn from(record: Record) -> Message {
        Message(record)
    }
}

impl Message {
    #[must_use]
    pub fn record(&self) -> &Record {
        &self.0
    }

    #[must_use]
    pub fn segments(&self) -> Vec<Segment> {
        Segmenter::segments(Lexemes::scanned(self.0.text()), self.0.uuid())
    }
}
