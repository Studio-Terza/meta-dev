mod answers;

use transcript_ports::Authorship;

use crate::anchor::Anchor;
use crate::{MechanicalLink, Message, Relation};
use answers::Answers;

pub struct Reply<'stream> {
    replying: &'stream Message,
    replied: &'stream Message,
}

struct OwnerReply<'stream> {
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
        match self.replying.record().author() {
            Authorship::Owner => OwnerReply { replying: self.replying, replied: self.replied }.answers(),
            Authorship::Lead | Authorship::Bystander => Answers::default(),
        }
    }
}

impl OwnerReply<'_> {
    fn answers(&self) -> Answers {
        match self.replied.record().author() {
            Authorship::Lead => Answers::between(self.replying, self.replied),
            Authorship::Owner | Authorship::Bystander => Answers::default(),
        }
    }
}
