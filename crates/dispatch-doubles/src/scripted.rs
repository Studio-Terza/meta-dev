use std::sync::Mutex;

use dispatch_ports::{
    AgentRun, DispatchFailure, DispatchFiller, Ended, Finished, Job, Output, Pid, Prompt, Reply, Started, Usage,
    dispatch_ops,
};
use invariants::Unpoisoned;
use port_core::Pass;

use crate::Count;

#[derive(Debug)]
pub struct Scripted {
    pid: Pid,
    reply: Reply,
    finishing: Option<Count>,
    polls: Mutex<Count>,
}

impl Scripted {
    #[must_use]
    pub fn replying(pid: Pid, reply: Reply, finishing: Option<Count>) -> Scripted {
        Scripted { pid, reply, finishing, polls: Mutex::default() }
    }
}

impl DispatchFiller for Scripted {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        job: &Job,
        _prompt: &Prompt,
    ) -> Result<AgentRun, DispatchFailure> {
        Ok(AgentRun::new(self.pid, *job.as_ref()))
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        _run: &mut AgentRun,
    ) -> Result<Option<Output>, DispatchFailure> {
        let mut polls = self.polls.lock().unpoisoned();
        *polls = polls.counted();
        let finished = self.finishing.filter(|finishing| *finishing == *polls);
        Ok(finished.map(|_| Output::Finished(Finished::new(self.reply.clone(), Usage::default()))))
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, _pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        Ok(None)
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, _pid: Pid) -> Result<Ended, DispatchFailure> {
        Ok(Ended::Absent)
    }
}
