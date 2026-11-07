use std::io;
use std::process::{Child, Command, Stdio};
use std::thread::JoinHandle;

use port_core::{Pass, Release, ReleaseReceipt, Released, Resource};

use crate::{Deadline, Pid};

#[derive(Debug, Resource)]
pub struct AgentRun {
    pid: Pid,
    deadline: Deadline,
    process: Option<Process>,
}

#[derive(Debug)]
pub(crate) struct Process {
    child: Child,
    reading: Option<Reading>,
}

pub(crate) type Reading = JoinHandle<io::Result<String>>;

#[derive(Debug, thiserror::Error)]
#[error("cannot kill the agent's process group: {0}")]
pub struct Unkilled(#[from] io::Error);

impl AgentRun {
    #[must_use]
    pub fn new(pid: Pid, deadline: Deadline) -> AgentRun {
        AgentRun { pid, deadline, process: None }
    }

    pub(crate) fn holding(self, child: Child, reading: Reading) -> AgentRun {
        AgentRun { process: Some(Process { child, reading: Some(reading) }), ..self }
    }

    pub(crate) fn read_out(&mut self) -> Option<Reading> {
        self.process.as_mut().and_then(|process| process.reading.take_if(|reading| reading.is_finished()))
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
                Command::new(Pid::KILL)
                    .args(Pid::SIGNAL)
                    .stderr(Stdio::null())
                    .arg(&group)
                    .status()
                    .and_then(|_signalled| process.child.wait())
            })
            .transpose()
            .map(|_reaped| pass.released())
            .map_err(Unkilled::from)
    }
}
