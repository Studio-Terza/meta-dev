mod chain;
mod reply;
mod resends;
mod turn;

use std::collections::BTreeSet;

use grammar::message::{Lexemes, Mark};
use transcript_ports::{Authorship, Skipped, Unbare, Uuid};

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

    pub fn all(&self) -> Result<Vec<MechanicalLink>, Unbare> {
        let replies = self.replies();
        let cites = self.messages.iter().map(|message| self.cites(message)).collect::<Result<Vec<_>, Unbare>>()?;
        let links: BTreeSet<MechanicalLink> = replies
            .iter()
            .map(Reply::link)
            .chain(self.turns().iter().flat_map(|turn| turn.answers().links()))
            .chain(cites.into_iter().flatten())
            .chain(Resends::links(self.messages))
            .collect();
        Ok(links.into_iter().collect())
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

    fn cites(&self, message: &Message) -> Result<Vec<MechanicalLink>, Unbare> {
        let cited = message
            .segments()
            .iter()
            .map(|segment| self.cited_by(segment, message))
            .collect::<Result<Vec<_>, Unbare>>()?;
        Ok(cited.into_iter().flatten().collect())
    }

    fn cited_by(&self, segment: &Segment, message: &Message) -> Result<Vec<MechanicalLink>, Unbare> {
        let text = segment.text(message);
        let uuids = Lexemes::scanned(&text)
            .into_iter()
            .filter(|lexeme| matches!(lexeme.mark(), Mark::Uuid))
            .map(|lexeme| Uuid::try_new(lexeme.span().slice(text.as_ref()).to_owned()))
            .collect::<Result<Vec<Uuid>, Unbare>>()?;
        Ok(uuids
            .into_iter()
            .filter(|cited| cited != message.record().uuid())
            .filter_map(|cited| self.chain.known(&cited))
            .map(|known| {
                MechanicalLink::new(
                    Anchor::Segment(segment.identity().clone()),
                    Relation::Cites,
                    Anchor::Message(known.record().uuid().clone()),
                )
            })
            .collect())
    }
}
