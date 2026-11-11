use crate::anchor::Anchor;
use crate::{MechanicalLink, Message, Relation};

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
}
