use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use port_core::Pass;

use crate::agent_run::Exchange;
use crate::{Agent, AgentRun, DispatchFailure, DispatchFiller, Ended, Job, Output, Pid, Prompt, Started, dispatch_ops};

#[derive(Clone, Debug)]
pub struct HeadlessClaude {
    scratch: PathBuf,
    agent: Agent,
}

impl HeadlessClaude {
    const LISTER: &'static str = "ps";
    const LISTING: [&'static str; 3] = ["-o", "lstart=", "-p"];
    const LOCALE: [&'static str; 2] = ["LC_ALL", "C"];

    #[must_use]
    pub fn new(scratch: PathBuf, agent: Agent) -> HeadlessClaude {
        HeadlessClaude { scratch, agent }
    }
}

impl DispatchFiller for HeadlessClaude {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        job: &Job,
        prompt: &Prompt,
    ) -> Result<AgentRun, DispatchFailure> {
        fs::create_dir_all(&self.scratch).map_err(DispatchFailure::Scratch)?;
        let (agent_in, prompting) = io::pipe().map_err(DispatchFailure::Pipe)?;
        let (replies, agent_out) = io::pipe().map_err(DispatchFailure::Pipe)?;
        let exchange = Exchange::begun(prompt, prompting, replies);
        Command::new(self.agent.program())
            .args(self.agent.arguments())
            .current_dir(&self.scratch)
            .process_group(0)
            .stdin(agent_in)
            .stdout(agent_out)
            .stderr(Stdio::null())
            .spawn()
            .map(|child| AgentRun::spawned(child, exchange, *job.as_ref()))
            .map_err(DispatchFailure::Spawn)
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        run: &mut AgentRun,
    ) -> Result<Option<Output>, DispatchFailure> {
        run.read_out().map(|exchange| Ok(Output::read(&exchange.joined()?))).transpose()
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        let [name, value] = Self::LOCALE;
        let listed = Command::new(Self::LISTER)
            .args(Self::LISTING)
            .arg(pid.listed())
            .env(name, value)
            .output()
            .map_err(DispatchFailure::Lookup)?;
        Ok(str::from_utf8(&listed.stdout).ok().and_then(|text| text.parse::<Started>().ok()))
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, pid: Pid) -> Result<Ended, DispatchFailure> {
        let status = Command::new(Pid::KILL)
            .args(Pid::SIGNAL)
            .arg(pid.group())
            .stderr(Stdio::null())
            .status()
            .map_err(DispatchFailure::Kill)?;
        Ok(Ended::from(status))
    }
}
