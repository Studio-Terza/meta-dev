mod boundary;

use port_core::Op;
use transcript_ports::SessionId;

#[derive(Clone, Debug, Op)]
pub struct EarlierLines(SessionId);

impl EarlierLines {
    #[must_use]
    pub fn within(session: SessionId) -> EarlierLines {
        EarlierLines(session)
    }
}
