mod boundary;

use port_core::Op;
use transcript_ports::{Instant, SessionId};

pub use boundary::{Quote, Said, TurnEnd, Unabridged};

#[derive(Clone, Copy, Debug, Op)]
pub struct OwnerHistory;

#[derive(Clone, Debug, Op)]
pub struct LastSaid(SessionId);

#[derive(Clone, Debug, Op)]
pub struct OwnerBefore(Instant);

#[derive(Clone, Debug, Op)]
pub struct TurnEnds {
    session: SessionId,
    from: Instant,
    until: Instant,
}

impl TurnEnds {
    #[must_use]
    pub fn between(session: SessionId, from: Instant, until: Instant) -> TurnEnds {
        TurnEnds { session, from, until }
    }
}

impl From<SessionId> for LastSaid {
    fn from(session: SessionId) -> LastSaid {
        LastSaid(session)
    }
}

impl From<Instant> for OwnerBefore {
    fn from(at: Instant) -> OwnerBefore {
        OwnerBefore(at)
    }
}
