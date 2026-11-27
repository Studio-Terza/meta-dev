use std::collections::VecDeque;
use std::sync::Mutex;

use invariants::{Ignoring, Unpoisoned};
use port_core::Pass;

use crate::{
    AgentRun, DispatchFailure, DispatchFiller, Ended, Ignored, Job, Output, Pid, Prompt, Started, dispatch_ops,
};

#[derive(Debug, Default)]
pub struct ReplayedDispatch {
    replies: Mutex<VecDeque<Option<Output>>>,
    running: Mutex<Vec<Replaying>>,
    prompts: Mutex<Vec<Prompt>>,
}

#[derive(Debug)]
struct Replaying {
    pid: Pid,
    reply: Option<Output>,
}

impl ReplayedDispatch {
    #[must_use]
    pub fn replaying(replies: impl IntoIterator<Item = Option<Output>>) -> ReplayedDispatch {
        ReplayedDispatch { replies: Mutex::new(replies.into_iter().collect()), ..ReplayedDispatch::default() }
    }

    #[must_use]
    pub fn prompts(&self) -> Vec<Prompt> {
        self.prompts.lock().unpoisoned().clone()
    }

    fn reply(&self, pid: Pid) -> Option<Output> {
        let running = self.running.lock().unpoisoned();
        running.iter().find(|replaying| replaying.pid == pid).and_then(|replaying| replaying.reply.clone())
    }
}

impl DispatchFiller for &ReplayedDispatch {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        job: &Job,
        prompt: &Prompt,
    ) -> Result<AgentRun, DispatchFailure> {
        self.prompts.lock().unpoisoned().push(prompt.clone());
        let reply = self.replies.lock().unpoisoned().pop_front().flatten();
        let mut running = self.running.lock().unpoisoned();
        let pid = Pid::new(u32::try_from(running.len()).ignoring(Ignored::ManyRuns).unwrap_or(u32::MAX));
        running.push(Replaying { pid, reply });
        Ok(AgentRun::new(pid, *job.as_ref()))
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        run: &mut AgentRun,
    ) -> Result<Option<Output>, DispatchFailure> {
        Ok(self.reply(run.pid()))
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, _pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        Ok(None)
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, _pid: Pid) -> Result<Ended, DispatchFailure> {
        Ok(Ended::Absent)
    }
}
