mod links;

use transcript_ports::{Gather, Line, Record, Skipped, Unrecognised};

use crate::{MechanicalLink, Message};
use links::Links;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stream {
    messages: Vec<Message>,
    skipped: Vec<Skipped>,
    unrecognised: Vec<Unrecognised>,
}

impl FromIterator<Line> for Stream {
    fn from_iter<Lines: IntoIterator<Item = Line>>(lines: Lines) -> Stream {
        lines.into_iter().fold(Stream::default(), |stream, line| line.gathered(stream))
    }
}

impl Gather for Stream {
    fn said(mut self, record: Record) -> Stream {
        self.messages.push(Message::from(record));
        self
    }

    fn skipped(mut self, skipped: Skipped) -> Stream {
        self.skipped.push(skipped);
        self
    }

    fn unrecognised(mut self, unrecognised: Unrecognised) -> Stream {
        self.unrecognised.push(unrecognised);
        self
    }
}

impl AsRef<[Message]> for Stream {
    fn as_ref(&self) -> &[Message] {
        &self.messages
    }
}

impl Stream {
    #[must_use]
    pub fn links(&self) -> Vec<MechanicalLink> {
        Links::over(&self.messages, &self.skipped).all()
    }

    #[must_use]
    pub fn unrecognised(&self) -> &[Unrecognised] {
        &self.unrecognised
    }
}
