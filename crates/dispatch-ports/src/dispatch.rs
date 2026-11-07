use port_core::{Held, port};

use crate::{AgentRun, DispatchFailure, Ended, Finished, Job, Pid, Prompt, Started};

#[port]
pub trait Dispatch {
    fn dispatch(&self, job: &Job, prompt: &Prompt) -> Result<Held<'_, AgentRun>, DispatchFailure>;

    fn output(&self, run: &mut AgentRun) -> Result<Option<Finished>, DispatchFailure>;

    fn started(&self, pid: Pid) -> Result<Option<Started>, DispatchFailure>;

    fn kill(&self, pid: Pid) -> Result<Ended, DispatchFailure>;
}
