use std::fs;
use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

use port_core::Pass;

use crate::{AgentRun, DispatchFailure, DispatchFiller, Finished, Job, Pid, Prompt, dispatch_ops};

#[derive(Clone, Debug)]
pub struct HeadlessClaude {
    scratch: PathBuf,
}

impl HeadlessClaude {
    const PROGRAM: &'static str = "claude";
    const ARGUMENTS: [&'static str; 9] =
        ["-p", "--model", "haiku", "--output-format", "json", "--setting-sources", "user", "--tools", ""];
}

impl From<PathBuf> for HeadlessClaude {
    fn from(scratch: PathBuf) -> HeadlessClaude {
        HeadlessClaude { scratch }
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
        let mut child = Command::new(Self::PROGRAM)
            .args(Self::ARGUMENTS)
            .current_dir(&self.scratch)
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(DispatchFailure::Spawn)?;
        let run = AgentRun::new(Pid::new(u64::from(child.id())), job.deadline());
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
}
