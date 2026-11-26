mod boundary;

use port_core::Op;
use transcript_ports::SessionId;

#[derive(Clone, Debug, Op)]
pub struct SessionEvents(SessionId);

impl From<SessionId> for SessionEvents {
    fn from(session: SessionId) -> SessionEvents {
        SessionEvents(session)
    }
}
