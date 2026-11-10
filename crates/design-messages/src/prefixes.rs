mod boundary;

use std::collections::{BTreeMap, BTreeSet};

use design_model::Prefix;
use port_core::Op;
use transcript_ports::Uuid;

#[derive(Clone, Debug, Op)]
pub struct Prefixes(BTreeSet<Uuid>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shortest(BTreeMap<Uuid, Prefix>);

impl FromIterator<Uuid> for Prefixes {
    fn from_iter<Messages: IntoIterator<Item = Uuid>>(messages: Messages) -> Prefixes {
        Prefixes(messages.into_iter().collect())
    }
}

impl Shortest {
    #[must_use]
    pub fn prefix(&self, message: &Uuid) -> Prefix {
        self.0.get(message).cloned().unwrap_or_else(|| Prefix::new(message.clone().into_inner()))
    }
}
