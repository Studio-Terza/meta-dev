mod reply;
mod resends;

use std::collections::{BTreeMap, BTreeSet};

use grammar::message::{Lexemes, Mark};
use transcript_ports::{Skipped, Uuid};

use crate::anchor::Anchor;
use crate::segment::Segment;
use crate::{MechanicalLink, Message, Relation};
use reply::Reply;
use resends::Resends;

pub struct Links<'stream> {
    messages: &'stream [Message],
    known: BTreeMap<&'stream Uuid, &'stream Message>,
    skipped: BTreeMap<&'stream Uuid, &'stream Uuid>,
}

impl<'stream> Links<'stream> {
    pub fn over(messages: &'stream [Message], skipped: &'stream [Skipped]) -> Links<'stream> {
        Links {
            messages,
            known: messages.iter().map(|message| (message.record().uuid(), message)).collect(),
            skipped: skipped.iter().filter_map(|line| line.parent().map(|parent| (line.uuid(), parent))).collect(),
        }
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
        std::iter::successors(message.record().parent(), |uuid| self.skipped.get(*uuid).copied())
            .take(self.skipped.len().saturating_add(1))
            .find_map(|uuid| self.known.get(uuid).copied())
    }

    fn replies(&self) -> Vec<Reply<'stream>> {
        self.messages
            .iter()
            .filter_map(|message| self.parent(message).map(|replied| Reply::new(message, replied)))
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
