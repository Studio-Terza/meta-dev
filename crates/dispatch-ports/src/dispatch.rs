use port_core::{Held, port};

use crate::{AgentRun, DispatchFailure, Finished, Job, Prompt};

#[port]
pub trait Dispatch {
    fn dispatch(&self, job: &Job, prompt: &Prompt) -> Result<Held<'_, AgentRun>, DispatchFailure>;

    fn output(&self, run: &mut AgentRun) -> Result<Option<Finished>, DispatchFailure>;
}
