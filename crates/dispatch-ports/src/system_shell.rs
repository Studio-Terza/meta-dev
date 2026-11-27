mod boundary;

use std::iter;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, SystemTime};

use port_core::Pass;

use crate::{Call, Deadline, DispatchFailure, Due, Pid, Ran, ShellFiller, Wall, shell_ops};

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemShell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Waited {
    Exited(ExitStatus),
    Overdue,
}

impl SystemShell {
    const POLL: Duration = Duration::from_millis(50);

    fn waited(child: &mut Child, deadline: Deadline) -> Result<Waited, DispatchFailure> {
        iter::repeat_with(|| SystemShell::checked(child, deadline))
            .find_map(Result::transpose)
            .unwrap_or(Ok(Waited::Overdue))
    }

    fn checked(child: &mut Child, deadline: Deadline) -> Result<Option<Waited>, DispatchFailure> {
        let exited = child.try_wait().map_err(DispatchFailure::Collect)?;
        let due = deadline.due(Wall::from(SystemTime::now()));
        Ok(exited.map(Waited::Exited).or_else(|| match due {
            Due::Passed => Some(Waited::Overdue),
            Due::Pending => {
                thread::sleep(SystemShell::POLL);
                None
            }
        }))
    }

    fn killed(child: &mut Child) -> Result<Ran, DispatchFailure> {
        Command::new(Pid::KILL)
            .args(Pid::SIGNAL)
            .arg(Pid::new(child.id()).group())
            .stderr(Stdio::null())
            .status()
            .and_then(|_signalled| child.wait())
            .map(|_reaped| Ran::Overdue)
            .map_err(DispatchFailure::Kill)
    }
}

impl ShellFiller for SystemShell {
    fn ran(&self, _pass: Pass<'_, shell_ops::Ran>, call: &Call) -> Result<Ran, DispatchFailure> {
        let mut child = Command::new(call.program())
            .args(call.arguments())
            .current_dir(call.directory())
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(DispatchFailure::Spawn)?;
        let reading = child.stdout.take().map(SystemShell::read);
        match SystemShell::waited(&mut child, call.deadline())? {
            Waited::Exited(status) => {
                let printed = SystemShell::collected(reading)?;
                Ok(if status.success() { Ran::Succeeded(printed) } else { Ran::Failed(printed) })
            }
            Waited::Overdue => SystemShell::killed(&mut child),
        }
    }
}
