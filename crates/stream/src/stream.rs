mod links;

use std::collections::BTreeSet;

use newtype::{Arbitrary, Display, Newtype, Wrapping};
use transcript_ports::{Gather, Line, Record, Skipped, Unbare, Unrecognised, Uuid};

use crate::{Earlier, MechanicalLink, Message};
use links::Links;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Earliest(usize);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Resumed {
    messages: Earliest,
    skipped: Earliest,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stream {
    messages: Vec<Message>,
    skipped: Vec<Skipped>,
    unrecognised: Vec<Unrecognised>,
    resumed: Resumed,
}

impl FromIterator<Line> for Stream {
    fn from_iter<Lines: IntoIterator<Item = Line>>(lines: Lines) -> Stream {
        Earlier::default().resumed(lines)
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
        self.skipped.extend(unrecognised.passing());
        self.unrecognised.push(unrecognised);
        self
    }
}

impl AsRef<[Message]> for Stream {
    fn as_ref(&self) -> &[Message] {
        self.messages.get(self.resumed.messages.into_inner()..).unwrap_or_default()
    }
}

impl Stream {
    pub(crate) fn after(messages: Vec<Message>, skipped: Vec<Skipped>) -> Stream {
        let resumed = Resumed { messages: Earliest::new(messages.len()), skipped: Earliest::new(skipped.len()) };
        Stream { messages, skipped, unrecognised: Vec::new(), resumed }
    }

    pub fn links(&self) -> Result<Vec<MechanicalLink>, Unbare> {
        let fresh: BTreeSet<&Uuid> = self.as_ref().iter().map(|message| message.record().uuid()).collect();
        Ok(Links::over(&self.messages, &self.skipped)
            .all()?
            .into_iter()
            .filter(|link| link.anchors().iter().any(|anchor| fresh.contains(anchor.message())))
            .collect())
    }

    #[must_use]
    pub fn skipped(&self) -> &[Skipped] {
        self.skipped.get(self.resumed.skipped.into_inner()..).unwrap_or_default()
    }

    #[must_use]
    pub fn unrecognised(&self) -> &[Unrecognised] {
        &self.unrecognised
    }
}
