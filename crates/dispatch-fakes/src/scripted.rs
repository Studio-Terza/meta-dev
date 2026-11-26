mod boundary;

use std::collections::{BTreeMap, VecDeque};
use std::sync::Mutex;

use invariants::Unpoisoned;
use port_core::Pass;

use boundary::Spawns;
use dispatch_ports::{
    AgentRun, Bell, ClockFiller, Deadline, DispatchFailure, DispatchFiller, Ended, Job, Model, Output, Pause, Pid,
    Prompt, Started, Wall, clock_ops, dispatch_ops,
};

#[derive(Clone, Debug)]
struct Living {
    pid: Pid,
    started: Started,
}

#[derive(Debug, Default)]
pub struct Scripted {
    replies: Mutex<VecDeque<Option<Output>>>,
    pending: Mutex<BTreeMap<Pid, Output>>,
    prompts: Mutex<Vec<Prompt>>,
    models: Mutex<Vec<Model>>,
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
        self.prompts.lock().unpoisoned().clone()
    }

    #[must_use]
    pub fn models(&self) -> Vec<Model> {
        self.models.lock().unpoisoned().clone()
    }

    #[must_use]
    pub fn kills(&self) -> Vec<Pid> {
        self.kills.lock().unpoisoned().clone()
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
        let mut prompts = self.prompts.lock().unpoisoned();
        prompts.push(prompt.clone());
        self.models.lock().unpoisoned().push(*AsRef::<Model>::as_ref(job));
        let pid = Spawns::pid(prompts.len());
        let reply = self.replies.lock().unpoisoned().pop_front().flatten();
        self.pending.lock().unpoisoned().extend(reply.map(|output| (pid, output)));
        Ok(AgentRun::new(pid, *AsRef::<Deadline>::as_ref(job)))
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        run: &mut AgentRun,
    ) -> Result<Option<Output>, DispatchFailure> {
        Ok(self.pending.lock().unpoisoned().remove(&run.pid()))
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        Ok(self.alive(pid).map(|living| living.started.clone()))
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, pid: Pid) -> Result<Ended, DispatchFailure> {
        self.kills.lock().unpoisoned().push(pid);
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
        *self.now.lock().unpoisoned()
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause, _bell: &Bell) -> Wall {
        let mut now = self.now.lock().unpoisoned();
        *now = now.after(pause).after(self.sleep);
        *now
    }
}
