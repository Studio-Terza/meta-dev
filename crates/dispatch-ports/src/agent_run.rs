mod boundary;

use std::process::{Child, Command, Stdio};

use port_core::{Pass, Release, ReleaseReceipt, Released, Resource};

use invariants::Ignoring;

use crate::{Bell, Deadline, DispatchFailure, Ignored, Output, Pid};
use boundary::Process;

pub(crate) use boundary::Exchange;

#[derive(Debug, Resource)]
pub struct AgentRun {
    pid: Pid,
    deadline: Deadline,
    bell: Bell,
    process: Option<Process>,
}

impl AgentRun {
    #[must_use]
    pub fn new(pid: Pid, deadline: Deadline) -> AgentRun {
        AgentRun { pid, deadline, bell: Bell::hung(), process: None }
    }

    pub(crate) fn spawned(child: Child, exchange: Exchange, bell: Bell, deadline: Deadline) -> AgentRun {
        let pid = Pid::new(child.id());
        AgentRun { bell, process: Some(Process::new(child, exchange)), ..AgentRun::new(pid, deadline) }
    }

    pub(crate) fn read_out(&mut self) -> Option<Result<Output, DispatchFailure>> {
        self.process.as_mut().and_then(Process::read_out)
    }

    #[must_use]
    pub fn pid(&self) -> Pid {
        self.pid
    }

    #[must_use]
    pub fn bell(&self) -> &Bell {
        &self.bell
    }
}

impl AsRef<Deadline> for AgentRun {
    fn as_ref(&self) -> &Deadline {
        &self.deadline
    }
}

impl Release for AgentRun {
    fn release(&mut self, pass: Pass<'_, Released<AgentRun>>) -> ReleaseReceipt {
        let group = self.pid.group();
        let reaped = self.process.take().map(|mut process| {
            Command::new(Pid::KILL)
                .args(Pid::SIGNAL)
                .stderr(Stdio::null())
                .arg(&group)
                .status()
                .and_then(|_signalled| process.child().wait())
        });
        reaped.transpose().ignoring(Ignored::Reaped);
        pass.released()
    }
}
