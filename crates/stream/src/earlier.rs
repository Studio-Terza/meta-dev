use transcript_ports::{Line, Record, Skipped};

use crate::{Message, Stream};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Earlier {
    messages: Vec<Message>,
    skipped: Vec<Skipped>,
}

impl Earlier {
    #[must_use]
    pub fn new(said: Vec<Record>, skipped: Vec<Skipped>) -> Earlier {
        Earlier { messages: said.into_iter().map(Message::from).collect(), skipped }
    }

    #[must_use]
    pub fn resumed(self, lines: impl IntoIterator<Item = Line>) -> Stream {
        lines.into_iter().fold(Stream::after(self.messages, self.skipped), |stream, line| line.gathered(stream))
    }
}
