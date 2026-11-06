mod reply;
mod resends;

use std::collections::{BTreeMap, BTreeSet};

use grammar::message::{Lexemes, Mark};
use transcript_ports::{Authorship, Uuid};

use crate::anchor::Anchor;
use crate::segment::Segment;
use crate::{MechanicalLink, Message, Relation};
use reply::Reply;
use resends::Resends;

pub struct Links<'stream> {
    messages: &'stream [Message],
    known: BTreeMap<&'stream Uuid, &'stream Message>,
}

impl<'stream> Links<'stream> {
    pub fn over(messages: &'stream [Message]) -> Links<'stream> {
        Links { messages, known: messages.iter().map(|message| (message.record().uuid(), message)).collect() }
    }

    pub fn all(&self) -> Vec<MechanicalLink> {
        let replies = self.replies();
        let links: BTreeSet<MechanicalLink> = replies
            .iter()
            .map(Reply::link)
            .chain(replies.iter().flat_map(|reply| reply.answers().links()))
            .chain(self.messages.iter().flat_map(|message| self.cites(message)))
            .chain(Resends::links(self.messages))
            .collect();
        links.into_iter().collect()
    }

    fn parent(&self, message: &Message) -> Option<&'stream Message> {
        message.record().parent().and_then(|parent| self.known.get(parent).copied())
    }

    fn replies(&self) -> Vec<Reply<'stream>> {
        self.messages
            .iter()
            .filter(|message| message.record().author() != Authorship::Bystander)
            .filter_map(|message| {
                std::iter::successors(self.parent(message), |ancestor| self.parent(ancestor))
                    .take(self.messages.len())
                    .find(|ancestor| ancestor.record().author() != Authorship::Bystander)
                    .map(|replied| Reply::new(message, replied))
            })
            .collect()
    }

    fn cites(&self, message: &Message) -> Vec<MechanicalLink> {
        message.segments().iter().flat_map(|segment| self.cited_by(segment, message)).collect()
    }

    fn cited_by(&self, segment: &Segment, message: &Message) -> Vec<MechanicalLink> {
        let text = segment.text(message);
        Lexemes::scanned(&text)
            .into_iter()
            .filter(|lexeme| matches!(lexeme.mark(), Mark::Uuid))
            .map(|lexeme| Uuid::new(lexeme.span().slice(text.as_ref()).to_owned()))
            .filter(|cited| cited != message.record().uuid())
            .filter(|cited| self.known.contains_key(cited))
            .map(|cited| {
                MechanicalLink::new(
                    Anchor::Segment(segment.identity().clone()),
                    Relation::Cites,
                    Anchor::Message(cited),
                )
            })
            .collect()
    }
}
