use grammar::message::Lexemes;
use transcript_ports::Record;

use crate::segmenter::Segmenter;
use crate::{Ordinal, Segment, SegmentId};

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
        Segmenter::parts(Lexemes::of(&self.0.text))
            .into_iter()
            .enumerate()
            .map(|(position, part)| Segment {
                id: SegmentId::new(self.0.uuid.clone(), Ordinal::new(position)),
                range: part.span,
                form: part.form,
            })
            .collect()
    }
}
