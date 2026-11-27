mod boundary;

use port_core::Op;
use transcript_ports::SessionId;

pub use boundary::{Quote, Said, Unabridged};

#[derive(Clone, Copy, Debug, Op)]
pub struct OwnerHistory;

#[derive(Clone, Debug, Op)]
pub struct LastSaid(SessionId);

impl From<SessionId> for LastSaid {
    fn from(session: SessionId) -> LastSaid {
        LastSaid(session)
    }
}
