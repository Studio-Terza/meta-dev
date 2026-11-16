mod boundary;

use std::collections::VecDeque;
use std::sync::{Mutex, PoisonError};

use port_core::Pass;

use crate::{
    AgentRun, Bell, ClockFiller, DispatchFailure, DispatchFiller, Ended, Job, Output, Pause, Pid, Prompt, Started,
    Wall, clock_ops, dispatch_ops,
};
use boundary::Spawns;

#[derive(Clone, Debug)]
struct Living {
    pid: Pid,
    started: Started,
}

#[derive(Debug, Default)]
pub struct Scripted {
    replies: Mutex<VecDeque<Option<Output>>>,
    current: Mutex<Option<Output>>,
    prompts: Mutex<Vec<Prompt>>,
    living: Vec<Living>,
    kills: Mutex<Vec<Pid>>,
}

impl Scripted {
    #[must_use]
    pub fn replying(replies: impl IntoIterator<Item = Option<Output>>) -> Scripted {
        Scripted { replies: Mutex::new(replies.into_iter().collect()), ..Scripted::default() }
    }

    #[must_use]
    pub fn living(self, pid: Pid, started: Started) -> Scripted {
        Scripted { living: vec![Living { pid, started }], ..self }
    }

    #[must_use]
    pub fn prompts(&self) -> Vec<Prompt> {
        self.prompts.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    #[must_use]
    pub fn kills(&self) -> Vec<Pid> {
        self.kills.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    fn alive(&self, pid: Pid) -> Option<&Living> {
        self.living.iter().find(|living| living.pid == pid)
    }
}

impl DispatchFiller for &Scripted {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        job: &Job,
        prompt: &Prompt,
    ) -> Result<AgentRun, DispatchFailure> {
        let mut prompts = self.prompts.lock().unwrap_or_else(PoisonError::into_inner);
        prompts.push(prompt.clone());
        let reply = self.replies.lock().unwrap_or_else(PoisonError::into_inner).pop_front().flatten();
        *self.current.lock().unwrap_or_else(PoisonError::into_inner) = reply;
        Ok(AgentRun::new(Spawns::pid(prompts.len()), *job.as_ref()))
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        _run: &mut AgentRun,
    ) -> Result<Option<Output>, DispatchFailure> {
        Ok(self.current.lock().unwrap_or_else(PoisonError::into_inner).take())
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        Ok(self.alive(pid).map(|living| living.started.clone()))
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, pid: Pid) -> Result<Ended, DispatchFailure> {
        self.kills.lock().unwrap_or_else(PoisonError::into_inner).push(pid);
        Ok(self.alive(pid).map_or(Ended::Absent, |_| Ended::Killed))
    }
}

#[derive(Debug)]
pub struct Stepped {
    now: Mutex<Wall>,
    sleep: Pause,
}

impl Stepped {
    #[must_use]
    pub fn starting(at: Wall, sleep: Pause) -> Stepped {
        Stepped { now: Mutex::new(at), sleep }
    }
}

impl ClockFiller for Stepped {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause, _bell: &Bell) -> Wall {
        let mut now = self.now.lock().unwrap_or_else(PoisonError::into_inner);
        *now = now.after(pause).after(self.sleep);
        *now
    }
}
