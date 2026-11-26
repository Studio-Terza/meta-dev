mod boundary;

use grammar_search::Terms;
use port_core::Op;
use transcript_ports::{Instant, SessionId};

pub use boundary::{Quote, Said, TurnEnd, Unabridged};

#[derive(Clone, Debug, Op)]
pub struct OwnerSaid {
    terms: Terms,
    before: Option<Instant>,
}

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

impl OwnerSaid {
    pub const MOST: usize = 60;
    const EACH: usize = 10;
    const SHARPEST: usize = 5;

    #[must_use]
    pub fn any(terms: Terms) -> OwnerSaid {
        OwnerSaid { terms, before: None }
    }

    #[must_use]
    pub fn before(terms: Terms, at: Instant) -> OwnerSaid {
        OwnerSaid { terms, before: Some(at) }
    }
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
