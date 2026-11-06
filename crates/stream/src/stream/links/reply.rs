mod answers;

use transcript_ports::Authorship;

use crate::anchor::Anchor;
use crate::{MechanicalLink, Message, Relation};
use answers::Answers;

pub struct Reply<'stream> {
    replying: &'stream Message,
    replied: &'stream Message,
}

impl<'stream> Reply<'stream> {
    pub fn new(replying: &'stream Message, replied: &'stream Message) -> Reply<'stream> {
        Reply { replying, replied }
    }

    pub fn link(&self) -> MechanicalLink {
        MechanicalLink::new(
            Anchor::Message(self.replying.record().uuid().clone()),
            Relation::RepliesTo,
            Anchor::Message(self.replied.record().uuid().clone()),
        )
    }

    pub fn answers(&self) -> Answers {
        let answering = Some(self.replying)
            .filter(|reply| reply.record().author() == Authorship::Owner)
            .filter(|_| self.replied.record().author() == Authorship::Lead);
        Answers::new(answering.map(Message::segments).unwrap_or_default(), self.replied.segments())
    }
}
