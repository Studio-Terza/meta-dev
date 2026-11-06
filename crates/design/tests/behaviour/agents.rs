use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};

use design::Agents;
use dispatch_ports::{
    AgentRun, ClockFiller, Deadline, DispatchFailure, DispatchFiller, Ended, Finished, Pause, Pid, Prompt, Scratch,
    Started, Wall, clock_ops, dispatch_ops,
};
use port_core::Pass;

#[derive(Clone, Debug)]
pub enum Script {
    Reply(String),
    Hang,
}

#[derive(Debug, Default)]
struct Stage {
    scripts: VecDeque<Script>,
    live: BTreeMap<Pid, Script>,
    prompts: Vec<Prompt>,
    starts: BTreeMap<Pid, Started>,
    killed: Vec<Pid>,
}

#[derive(Clone, Debug, Default)]
pub struct Scripted(Arc<Mutex<Stage>>);

#[derive(Clone, Debug)]
pub struct Stopwatch {
    now: Arc<Mutex<Wall>>,
    sleep: Pause,
}

impl Script {
    fn finished(self) -> Option<Finished> {
        match self {
            Script::Reply(text) => Scripted::finished(&text),
            Script::Hang => None,
        }
    }
}

impl Scripted {
    pub const SPAWNED: u32 = 70_000;

    pub fn replying(scripts: impl IntoIterator<Item = Script>) -> Scripted {
        let scripted = Scripted::default();
        scripted.staged(|stage| stage.scripts.extend(scripts));
        scripted
    }

    pub fn running(self, pid: Pid, started: Option<Started>) -> Scripted {
        self.staged(|stage| stage.starts.extend(started.map(|started| (pid, started))));
        self
    }

    pub fn prompts(&self) -> Vec<Prompt> {
        self.staged(|stage| stage.prompts.clone())
    }

    pub fn killed(&self) -> Vec<Pid> {
        self.staged(|stage| stage.killed.clone())
    }

    pub fn started(day: u8) -> Option<Started> {
        format!("Fri Nov  {day} 01:16:00 2026").parse().ok()
    }

    fn finished(reply: &str) -> Option<Finished> {
        let usage = serde_json::json!({
            "input_tokens": 11,
            "output_tokens": 7,
            "cache_creation_input_tokens": 3,
            "cache_read_input_tokens": 5
        });
        serde_json::json!({ "subtype": "success", "result": reply, "usage": usage }).to_string().parse().ok()
    }

    fn staged<Answer>(&self, act: impl FnOnce(&mut Stage) -> Answer) -> Answer {
        act(&mut self.0.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

impl DispatchFiller for Scripted {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        prompt: &Prompt,
        deadline: Deadline,
        _scratch: &Scratch,
    ) -> Result<AgentRun, DispatchFailure> {
        self.staged(|stage| {
            let pid = Pid::new(Self::SPAWNED + u32::try_from(stage.prompts.len()).unwrap_or_default());
            stage.prompts.push(prompt.clone());
            let script = stage.scripts.pop_front().unwrap_or(Script::Hang);
            stage.live.insert(pid, script);
            stage.starts.extend(Self::started(6).map(|started| (pid, started)));
            Ok(AgentRun::new(pid, deadline))
        })
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        run: &mut AgentRun,
    ) -> Result<Option<Finished>, DispatchFailure> {
        let script = self.staged(|stage| stage.live.get(&run.pid()).cloned());
        Ok(script.and_then(Script::finished))
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        Ok(self.staged(|stage| stage.starts.get(&pid).cloned()))
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, pid: Pid) -> Result<Ended, DispatchFailure> {
        self.staged(|stage| stage.killed.push(pid));
        Ok(Ended::Killed)
    }
}

impl Stopwatch {
    pub const EPOCH: i64 = 1_000_000;

    pub fn awake() -> Stopwatch {
        Stopwatch::sleeping(Pause::new(0))
    }

    pub fn sleeping(sleep: Pause) -> Stopwatch {
        Stopwatch { now: Arc::new(Mutex::new(Wall::new(Self::EPOCH))), sleep }
    }
}

impl ClockFiller for Stopwatch {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause) -> Wall {
        let mut now = self.now.lock().unwrap_or_else(PoisonError::into_inner);
        *now = now.after(pause).after(self.sleep);
        *now
    }
}

pub struct Idle;

impl Idle {
    pub fn agents() -> Agents<Scripted, Stopwatch> {
        Agents::new(Scripted::default(), Stopwatch::awake())
    }
}
