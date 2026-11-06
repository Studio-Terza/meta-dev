use std::collections::BTreeMap;

use transcript_ports::{Authorship, Body, Uuid};

use crate::anchor::Anchor;
use crate::{MechanicalLink, Message, Relation};

#[derive(Debug)]
pub struct Resends;

impl Resends {
    pub fn links(messages: &[Message]) -> Vec<MechanicalLink> {
        messages
            .iter()
            .map(Message::record)
            .filter(|record| *record.text() != Body::default())
            .fold(BTreeMap::<(Authorship, &Body), Vec<&Uuid>>::new(), |mut sent, record| {
                sent.entry((record.author(), record.text())).or_default().push(record.uuid());
                sent
            })
            .values()
            .filter_map(|uuids| uuids.split_first())
            .flat_map(|(first, later)| later.iter().map(|resent| Resends::link(resent, first)))
            .collect()
    }

    fn link(resent: &Uuid, first: &Uuid) -> MechanicalLink {
        MechanicalLink::new(Anchor::Message(resent.clone()), Relation::Resends, Anchor::Message(first.clone()))
    }
}
