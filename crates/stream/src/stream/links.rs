mod chain;
mod reply;
mod resends;
mod turn;

use std::collections::BTreeSet;

use grammar::message::Lexemes;
use transcript_ports::{Authorship, Skipped, Uuid};

use crate::anchor::Anchor;
use crate::segment::Segment;
use crate::{MechanicalLink, Message, Relation};
use chain::Chain;
use reply::Reply;
use resends::Resends;
use turn::Turn;

pub struct Links<'stream> {
    messages: &'stream [Message],
    chain: Chain<'stream>,
}

impl<'stream> Links<'stream> {
    pub fn over(messages: &'stream [Message], skipped: &'stream [Skipped]) -> Links<'stream> {
        Links { messages, chain: Chain::over(messages, skipped) }
    }

    pub fn all(&self) -> Vec<MechanicalLink> {
        let replies = self.replies();
        let links: BTreeSet<MechanicalLink> = replies
            .iter()
            .map(Reply::link)
            .chain(self.turns().iter().flat_map(|turn| turn.answers().links()))
            .chain(self.messages.iter().flat_map(|message| self.cites(message)))
            .chain(Resends::links(self.messages))
            .collect();
        links.into_iter().collect()
    }

    fn replies(&self) -> Vec<Reply<'stream>> {
        self.messages
            .iter()
            .filter_map(|message| self.chain.parent(message).map(|replied| Reply::new(message, replied)))
            .collect()
    }

    fn turns(&self) -> Vec<Turn<'stream>> {
        self.messages
            .iter()
            .filter(|message| message.record().author() == Authorship::Owner)
            .map(|owner| self.chain.turn(owner))
            .collect()
    }

    fn cites(&self, message: &Message) -> Vec<MechanicalLink> {
        message.segments().iter().flat_map(|segment| self.cited_by(segment, message)).collect()
    }

    fn cited_by(&self, segment: &Segment, message: &Message) -> Vec<MechanicalLink> {
        let text = segment.text(message);
        Lexemes::uuids(&text)
            .into_iter()
            .map(Uuid::from)
            .filter(|cited| cited != message.record().uuid())
            .filter_map(|cited| self.chain.known(&cited))
            .map(|known| {
                MechanicalLink::new(
                    Anchor::Segment(segment.identity().clone()),
                    Relation::Cites,
                    Anchor::Message(known.record().uuid().clone()),
                )
            })
            .collect()
    }
}
