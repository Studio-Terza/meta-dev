mod boundary;

use grammar_search::Terms;
use port_core::Op;
use transcript_ports::SessionId;

pub use boundary::{Quote, Said};

#[derive(Clone, Debug, Op)]
pub struct OwnerSaid(Terms);

#[derive(Clone, Debug, Op)]
pub struct LastSaid(SessionId);

impl OwnerSaid {
    pub const MOST: usize = 60;
    const EACH: usize = 10;
    const SHARPEST: usize = 5;

    #[must_use]
    pub fn any(terms: Terms) -> OwnerSaid {
        OwnerSaid(terms)
    }
}

impl From<SessionId> for LastSaid {
    fn from(session: SessionId) -> LastSaid {
        LastSaid(session)
    }
}
