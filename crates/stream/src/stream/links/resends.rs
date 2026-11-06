use std::collections::BTreeMap;

use transcript_ports::{Authorship, Body, Uuid};

use crate::anchor::Anchor;
use crate::{MechanicalLink, Message, Relation};

#[derive(Debug)]
pub struct Resends;

impl Resends {
    pub fn links(messages: &[Message]) -> Vec<MechanicalLink> {
        let mut first: BTreeMap<(Authorship, &Body), &Uuid> = BTreeMap::new();
        messages
            .iter()
            .map(Message::record)
            .filter(|record| *record.text() != Body::default())
            .filter_map(|record| {
                let earliest = *first.entry((record.author(), record.text())).or_insert(record.uuid());
                Some(earliest).filter(|earlier| *earlier != record.uuid()).map(|earlier| {
                    MechanicalLink::new(
                        Anchor::Message(record.uuid().clone()),
                        Relation::Resends,
                        Anchor::Message(earlier.clone()),
                    )
                })
            })
            .collect()
    }
}
