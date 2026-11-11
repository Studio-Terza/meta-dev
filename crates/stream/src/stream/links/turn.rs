mod answers;

use crate::Message;
use answers::Answers;

pub struct Turn<'stream> {
    owner: &'stream Message,
    led: Vec<&'stream Message>,
}

impl<'stream> Turn<'stream> {
    pub fn closed_by(owner: &'stream Message, led: Vec<&'stream Message>) -> Turn<'stream> {
        Turn { owner, led }
    }

    pub fn answers(&self) -> Answers {
        Answers::between(self.owner, self.led.iter().rev().flat_map(|message| message.segments()).collect())
    }
}
