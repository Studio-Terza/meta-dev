use std::sync::Mutex;

use invariants::Unpoisoned;
use port_core::Pass;
use serde::Deserialize;

use crate::{
    AgentRun, Count, DispatchFailure, DispatchFiller, Ended, Finished, Job, Output, Pid, Prompt, Reply, Started, Usage,
    dispatch_ops,
};

#[derive(Debug)]
pub struct Scripted {
    pid: Pid,
    reply: Reply,
    finishing: Option<Count>,
    ringing: Ringing,
    polls: Mutex<Count>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum Ringing {
    #[default]
    Quiet,
    Early,
}

impl Scripted {
    #[must_use]
    pub fn replying(pid: Pid, reply: Reply, finishing: Option<Count>, ringing: Ringing) -> Scripted {
        Scripted { pid, reply, finishing, ringing, polls: Mutex::default() }
    }
}

impl DispatchFiller for Scripted {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        job: &Job,
        _prompt: &Prompt,
    ) -> Result<AgentRun, DispatchFailure> {
        let run = AgentRun::new(self.pid, *job.as_ref());
        let early = (self.ringing == Ringing::Early).then(|| run.bell().ringer());
        early.map_or_default(|ringer| ringer.ring());
        Ok(run)
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
