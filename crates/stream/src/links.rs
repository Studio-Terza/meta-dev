mod reply;

use std::collections::{BTreeMap, BTreeSet};

use grammar::message::{Lexemes, Mark};
use transcript_ports::{Authorship, Body, Uuid};

use crate::{Anchor, MechanicalLink, Message, Relation, Segment};
use reply::Reply;

pub struct Links<'stream> {
    messages: &'stream [Message],
    known: BTreeMap<&'stream Uuid, &'stream Message>,
}

impl<'stream> Links<'stream> {
    pub fn of(messages: &'stream [Message]) -> Links<'stream> {
        Links { messages, known: messages.iter().map(|message| (&message.record().uuid, message)).collect() }
    }

    pub fn all(&self) -> Vec<MechanicalLink> {
        let replies = self.replies();
        let links: BTreeSet<MechanicalLink> = replies
            .iter()
            .map(Reply::link)
            .chain(replies.iter().flat_map(Reply::answers))
            .chain(self.messages.iter().flat_map(|message| self.cites(message)))
            .chain(self.resends())
            .collect();
        links.into_iter().collect()
    }

    fn parent(&self, message: &Message) -> Option<&'stream Message> {
        message.record().parent.as_ref().and_then(|parent| self.known.get(parent).copied())
    }

    fn replies(&self) -> Vec<Reply<'stream>> {
        self.messages
            .iter()
            .filter(|message| message.record().author != Authorship::Bystander)
            .filter_map(|message| {
                std::iter::successors(self.parent(message), |ancestor| self.parent(ancestor))
                    .take(self.messages.len())
                    .find(|ancestor| ancestor.record().author != Authorship::Bystander)
                    .map(|replied| Reply::new(message, replied))
            })
            .collect()
    }

    fn cites(&self, message: &Message) -> Vec<MechanicalLink> {
        message.segments().iter().flat_map(|segment| self.cited_by(segment, message)).collect()
    }

    fn cited_by(&self, segment: &Segment, message: &Message) -> Vec<MechanicalLink> {
        let text = segment.text(message);
        Lexemes::of(&text)
            .into_iter()
            .filter(|lexeme| matches!(lexeme.mark, Mark::Uuid))
            .map(|lexeme| Uuid::new(lexeme.span.slice(text.as_ref()).to_owned()))
            .filter(|cited| *cited != message.record().uuid)
            .filter(|cited| self.known.contains_key(cited))
            .map(|cited| MechanicalLink {
                from: Anchor::Segment(segment.id.clone()),
                relation: Relation::Cites,
                to: Anchor::Message(cited),
            })
            .collect()
    }

    fn resends(&self) -> Vec<MechanicalLink> {
        let mut first: BTreeMap<(Authorship, &Body), &Uuid> = BTreeMap::new();
        self.messages
            .iter()
            .map(Message::record)
            .filter(|record| record.text.as_ref().chars().any(char::is_alphanumeric))
            .filter_map(|record| {
                let earliest = *first.entry((record.author, &record.text)).or_insert(&record.uuid);
                (*earliest != record.uuid).then(|| MechanicalLink {
                    from: Anchor::Message(record.uuid.clone()),
                    relation: Relation::Resends,
                    to: Anchor::Message(earliest.clone()),
                })
            })
            .collect()
    }
}
