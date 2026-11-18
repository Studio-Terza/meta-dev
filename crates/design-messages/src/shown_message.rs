mod boundary;

use port_core::Op;
use transcript_ports::Uuid;

#[derive(Clone, Debug, Op)]
pub struct ShownMessage(Uuid);

impl ShownMessage {
    #[must_use]
    pub fn named(uuid: Uuid) -> ShownMessage {
        ShownMessage(uuid)
    }
}
