use std::fs;
use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

use port_core::Pass;

use crate::{
    Agent, AgentRun, DispatchFailure, DispatchFiller, Ended, Finished, Job, Pid, Prompt, Started, dispatch_ops,
};

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
        let mut child = Command::new(self.agent.program())
            .args(self.agent.arguments())
            .current_dir(&self.scratch)
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(DispatchFailure::Spawn)?;
        let run = AgentRun::new(Pid::new(child.id()), *job.as_ref());
        let streams = child.stdin.take().zip(child.stdout.take());
        let (mut stdin, mut stdout) = streams.ok_or(DispatchFailure::Unpiped)?;
        let reading = thread::spawn(move || {
            let mut read = String::new();
            stdout.read_to_string(&mut read).map(|_length| read)
        });
        let held = run.holding(child, reading);
        stdin.write_all(prompt.as_ref()).map_err(DispatchFailure::Prompt)?;
        drop(stdin);
        Ok(held)
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        run: &mut AgentRun,
    ) -> Result<Option<Finished>, DispatchFailure> {
        run.read_out()
            .map(|reading| {
                let read = reading.join().map_err(|_panicked| DispatchFailure::Unjoined)?;
                Ok(read.map_err(DispatchFailure::Collect)?.parse::<Finished>()?)
            })
            .transpose()
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
