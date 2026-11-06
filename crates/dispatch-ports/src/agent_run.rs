use std::io;
use std::process::{Child, Command};
use std::thread::JoinHandle;

use port_core::{Pass, Release, ReleaseReceipt, Released, Resource};

use crate::{Deadline, DispatchFailure, Finished, Pid};

#[derive(Debug, Resource)]
pub struct AgentRun {
    pid: Pid,
    deadline: Deadline,
    process: Option<Process>,
}

#[derive(Debug)]
pub(crate) struct Process {
    child: Child,
    reading: JoinHandle<Result<Finished, DispatchFailure>>,
}

#[derive(Debug, thiserror::Error)]
#[error("cannot kill the agent's process group: {0}")]
pub struct Unkilled(#[from] io::Error);

impl Process {
    pub(crate) fn new(child: Child, reading: JoinHandle<Result<Finished, DispatchFailure>>) -> Process {
        Process { child, reading }
    }

    fn collected(self) -> Result<Finished, DispatchFailure> {
        self.reading.join().map_err(|_panicked| DispatchFailure::Unjoined)?
    }
}

impl AgentRun {
    const KILL: &'static str = "kill";
    const SIGNAL: [&'static str; 2] = ["-KILL", "--"];

    #[must_use]
    pub fn new(pid: Pid, deadline: Deadline) -> AgentRun {
        AgentRun { pid, deadline, process: None }
    }

    pub(crate) fn holding(self, process: Process) -> AgentRun {
        AgentRun { process: Some(process), ..self }
    }

    pub(crate) fn child(&mut self) -> Option<&mut Child> {
        self.process.as_mut().map(|process| &mut process.child)
    }

    pub(crate) fn collected(&mut self) -> Result<Option<Finished>, DispatchFailure> {
        self.process.take().map(Process::collected).transpose()
    }

    #[must_use]
    pub fn pid(&self) -> Pid {
        self.pid
    }

    #[must_use]
    pub fn deadline(&self) -> Deadline {
        self.deadline
    }
}

impl Release for AgentRun {
    type Failure = Unkilled;

    fn release(&mut self, pass: Pass<'_, Released<AgentRun>>) -> Result<ReleaseReceipt, Unkilled> {
        let group = self.pid.group();
        self.process
            .take()
            .map(|mut process| {
                Command::new(Self::KILL)
                    .args(Self::SIGNAL)
                    .arg(&group)
                    .status()
                    .and_then(|_signalled| process.child.wait())
            })
            .transpose()
            .map(|_reaped| pass.released())
            .map_err(Unkilled::from)
    }
}
