use port_core::{Held, port};

use crate::{AgentRun, Deadline, DispatchFailure, Ended, Finished, Pid, Prompt, Scratch, Started};

#[port]
pub trait Dispatch {
    fn dispatch(
        &self,
        prompt: &Prompt,
        deadline: Deadline,
        scratch: &Scratch,
    ) -> Result<Held<'_, AgentRun>, DispatchFailure>;

    fn output(&self, run: &mut AgentRun) -> Result<Option<Finished>, DispatchFailure>;

    fn started(&self, pid: Pid) -> Result<Option<Started>, DispatchFailure>;

    fn kill(&self, pid: Pid) -> Result<Ended, DispatchFailure>;
}
