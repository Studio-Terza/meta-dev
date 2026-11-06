use transcript_ports::Record;

use crate::links::Links;
use crate::{MechanicalLink, Message};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stream {
    messages: Vec<Message>,
}

impl FromIterator<Record> for Stream {
    fn from_iter<Records: IntoIterator<Item = Record>>(records: Records) -> Stream {
        Stream { messages: records.into_iter().map(Message::from).collect() }
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
        Links::of(&self.messages).all()
    }
}
