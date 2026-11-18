mod boundary;

use std::collections::BTreeSet;

use port_core::Op;
use transcript_ports::Uuid;

#[derive(Clone, Debug, Op)]
pub struct Covered(BTreeSet<Uuid>);

impl FromIterator<Uuid> for Covered {
    fn from_iter<Messages: IntoIterator<Item = Uuid>>(messages: Messages) -> Covered {
        Covered(messages.into_iter().collect())
    }
}
