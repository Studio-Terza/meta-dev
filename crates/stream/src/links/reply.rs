use transcript_ports::Authorship;

use crate::{Anchor, MechanicalLink, Message, Relation};

pub struct Reply<'stream> {
    from: &'stream Message,
    to: &'stream Message,
}

impl<'stream> Reply<'stream> {
    pub fn new(from: &'stream Message, to: &'stream Message) -> Reply<'stream> {
        Reply { from, to }
    }

    pub fn link(&self) -> MechanicalLink {
        MechanicalLink {
            from: Anchor::Message(self.from.record().uuid.clone()),
            relation: Relation::RepliesTo,
            to: Anchor::Message(self.to.record().uuid.clone()),
        }
    }

    pub fn answers(&self) -> Vec<MechanicalLink> {
        let asked = self.to.segments();
        Some(self)
            .filter(|reply| reply.from.record().author == Authorship::Owner)
            .filter(|reply| reply.to.record().author == Authorship::Lead)
            .into_iter()
            .flat_map(|reply| reply.from.segments())
            .filter_map(|answer| {
                answer.form.number().and_then(|_| asked.iter().find(|item| item.form == answer.form)).map(|item| {
                    MechanicalLink {
                        from: Anchor::Segment(answer.id.clone()),
                        relation: Relation::Answers,
                        to: Anchor::Segment(item.id.clone()),
                    }
                })
            })
            .collect()
    }
}
