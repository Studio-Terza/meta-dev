use std::collections::BTreeMap;

use transcript_ports::{Authorship, Skipped, Uuid};

use super::reply::Reply;
use super::turn::Turn;
use crate::Message;

pub struct Chain<'stream> {
    known: BTreeMap<&'stream Uuid, &'stream Message>,
    skipped: BTreeMap<&'stream Uuid, &'stream Uuid>,
}

impl<'stream> Chain<'stream> {
    pub fn over(messages: &'stream [Message], skipped: &'stream [Skipped]) -> Chain<'stream> {
        Chain {
            known: messages.iter().map(|message| (message.record().uuid(), message)).collect(),
            skipped: skipped.iter().filter_map(|line| line.parent().map(|parent| (line.uuid(), parent))).collect(),
        }
    }

    pub fn replied(&self, message: &'stream Message) -> Option<Reply<'stream>> {
        std::iter::successors(message.record().parent(), |uuid| self.skipped.get(*uuid).copied())
            .take(self.skipped.len().saturating_add(1))
            .find_map(|uuid| self.known.get(uuid).copied())
            .map(|parent| Reply::new(message, parent))
    }

    pub fn turn(&self, owner: &'stream Message) -> Turn<'stream> {
        Turn::closed_by(
            owner,
            std::iter::successors(self.replied(owner), |reply| self.replied(reply.replied()))
                .take(self.known.len())
                .map(|reply| reply.replied())
                .take_while(|message| message.record().author() == Authorship::Lead)
                .collect(),
        )
    }

    pub fn known(&self, uuid: &Uuid) -> Option<&'stream Message> {
        self.known.get(uuid).copied()
    }
}
