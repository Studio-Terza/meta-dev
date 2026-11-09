mod boundary;

use std::process::{Child, Command, Stdio};

use port_core::{Pass, Release, ReleaseReceipt, Released, Resource};

use crate::{Deadline, DispatchFailure, Output, Pid};
use boundary::Process;

pub(crate) use boundary::Exchange;

#[derive(Debug, Resource)]
pub struct AgentRun {
    pid: Pid,
    deadline: Deadline,
    process: Option<Process>,
}

impl AgentRun {
    #[must_use]
    pub fn new(pid: Pid, deadline: Deadline) -> AgentRun {
        AgentRun { pid, deadline, process: None }
    }

    pub(crate) fn spawned(child: Child, exchange: Exchange, deadline: Deadline) -> AgentRun {
        AgentRun { pid: Pid::new(child.id()), deadline, process: Some(Process::new(child, exchange)) }
    }

    pub(crate) fn read_out(&mut self) -> Option<Result<Output, DispatchFailure>> {
        self.process.as_mut().and_then(Process::read_out)
    }

    #[must_use]
    pub fn pid(&self) -> Pid {
        self.pid
    }
}

impl AsRef<Deadline> for AgentRun {
    fn as_ref(&self) -> &Deadline {
        &self.deadline
    }
}

impl Release for AgentRun {
    type Failure = DispatchFailure;

    fn release(&mut self, pass: Pass<'_, Released<AgentRun>>) -> Result<ReleaseReceipt, DispatchFailure> {
        let group = self.pid.group();
        self.process
            .take()
            .map(|mut process| {
                Command::new(Pid::KILL)
                    .args(Pid::SIGNAL)
                    .stderr(Stdio::null())
                    .arg(&group)
                    .status()
                    .and_then(|_signalled| process.child().wait())
            })
            .transpose()
            .map(|_reaped| pass.released())
            .map_err(DispatchFailure::Kill)
    }
}
