mod boundary;

use dispatch_ports::{Clock, Dispatch, Model, Prompt, Timeout};
use transcript_ports::Body;

use super::super::structure::Crew;
use crate::cli::failure::Failure;
use boundary::Waiting;
pub use boundary::{Snapshot, Stall};

impl Snapshot {
    const TIMEOUT: Timeout = Timeout::new(60);

    pub fn stalled<D: Dispatch, C: Clock>(&self, said: &Body, crew: &Crew<'_, D, C>) -> Result<Option<Stall>, Failure> {
        self.overrun(crew.now()).map_or_else(
            || {
                let reply = crew.replied(&Prompt::from(&self.asking(said)), Self::TIMEOUT, Model::Haiku)?;
                Ok(reply.as_ref().and_then(Waiting::read).and_then(Waiting::deadlocked))
            },
            |overrun| Ok(Some(overrun)),
        )
    }
}
