use std::fs;
use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::thread;

use port_core::Pass;

use crate::agent_run::Process;
use crate::{
    AgentRun, Deadline, DispatchFailure, DispatchFiller, Ended, Finished, Pid, Prompt, Scratch, Started, dispatch_ops,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct HeadlessClaude;

impl HeadlessClaude {
    const PROGRAM: &'static str = "claude";
    const ARGUMENTS: [&'static str; 9] =
        ["-p", "--model", "haiku", "--output-format", "json", "--setting-sources", "user", "--tools", ""];
    const KILL: &'static str = "kill";
    const SIGNAL: [&'static str; 2] = ["-KILL", "--"];
    const LISTER: &'static str = "ps";
    const LISTING: [&'static str; 3] = ["-o", "lstart=", "-p"];
    const LOCALE: [&'static str; 2] = ["LC_ALL", "C"];
}

impl DispatchFiller for HeadlessClaude {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        prompt: &Prompt,
        deadline: Deadline,
        scratch: &Scratch,
    ) -> Result<AgentRun, DispatchFailure> {
        fs::create_dir_all(scratch).map_err(DispatchFailure::Scratch)?;
        let mut child = Command::new(Self::PROGRAM)
            .args(Self::ARGUMENTS)
            .current_dir(scratch)
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(DispatchFailure::Spawn)?;
        let run = AgentRun::new(Pid::new(child.id()), deadline);
        let streams = child.stdin.take().zip(child.stdout.take());
        let reading = streams.map(|(stdin, mut stdout)| {
            let reading = thread::spawn(move || {
                let mut read = String::new();
                stdout.read_to_string(&mut read).map_err(DispatchFailure::Collect)?;
                Ok(read.parse::<Finished>()?)
            });
            (stdin, reading)
        });
        let (mut stdin, reading) = reading.ok_or(DispatchFailure::Unpiped)?;
        let held = run.holding(Process::new(child, reading));
        stdin.write_all(prompt.as_ref().as_bytes()).map_err(DispatchFailure::Prompt)?;
        Ok(held)
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        run: &mut AgentRun,
    ) -> Result<Option<Finished>, DispatchFailure> {
        let exited = run.child().map(Child::try_wait).transpose().map_err(DispatchFailure::Poll)?.flatten();
        exited.map_or(Ok(None), |_status| run.collected())
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        let [name, value] = Self::LOCALE;
        let listed = Command::new(Self::LISTER)
            .args(Self::LISTING)
            .arg(pid.listed())
            .env(name, value)
            .output()
            .map_err(DispatchFailure::Lookup)?;
        Ok(String::from_utf8(listed.stdout).ok().and_then(|text| text.parse::<Started>().ok()))
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, pid: Pid) -> Result<Ended, DispatchFailure> {
        let status =
            Command::new(Self::KILL).args(Self::SIGNAL).arg(pid.group()).status().map_err(DispatchFailure::Kill)?;
        Ok(Ended::from(status))
    }
}
