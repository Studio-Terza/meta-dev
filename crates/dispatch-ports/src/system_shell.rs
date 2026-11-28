use std::io::{self, Read, Write};
use std::iter;
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use invariants::Ignoring;
use port_core::{Pass, Real};
use timer_ports::{Deadline, Due, Wall};

use crate::ignored::Ignored;
use crate::{Call, DispatchFailure, Printed, Ran, ShellFiller, shell_ops};

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemShell;

impl Real for SystemShell {}

#[derive(Debug)]
struct Running {
    child: Child,
    deadline: Deadline,
    reading: Option<JoinHandle<io::Result<String>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Waited {
    Exited(ExitStatus),
    Overdue,
}

impl SystemShell {
    const POLL: Duration = Duration::from_millis(50);
    const KILL: &'static str = "kill";
    const SIGNAL: [&'static str; 2] = ["-KILL", "--"];

    fn started(call: &Call) -> Result<Running, DispatchFailure> {
        let input = call.input().map_or_else(Stdio::null, |_fed| Stdio::piped());
        let mut child = Command::new(call.program())
            .args(call.arguments())
            .current_dir(call.directory())
            .process_group(0)
            .stdin(input)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(DispatchFailure::Spawn)?;
        call.input()
            .zip(child.stdin.take())
            .into_iter()
            .for_each(|(fed, stdin)| SystemShell::fed(fed.to_owned(), stdin));
        let reading = child.stdout.take().map(SystemShell::read);
        Ok(Running { child, deadline: call.deadline(), reading })
    }

    fn fed(input: String, mut stdin: ChildStdin) {
        thread::spawn(move || stdin.write_all(input.as_bytes()).ignoring(Ignored::Unfed));
    }

    fn read(mut stdout: ChildStdout) -> JoinHandle<io::Result<String>> {
        thread::spawn(move || {
            let mut text = String::new();
            stdout.read_to_string(&mut text).map(|_read| text)
        })
    }

    fn awaited(running: Vec<Result<Running, DispatchFailure>>) -> Vec<Result<Ran, DispatchFailure>> {
        running.into_iter().map(|started| started.and_then(SystemShell::finished)).collect()
    }

    fn finished(mut running: Running) -> Result<Ran, DispatchFailure> {
        match SystemShell::waited(&mut running.child, running.deadline)? {
            Waited::Exited(status) => {
                let printed = SystemShell::collected(running.reading)?;
                Ok(if status.success() { Ran::Succeeded(printed) } else { Ran::Failed(printed) })
            }
            Waited::Overdue => SystemShell::killed(&mut running.child),
        }
    }

    fn waited(child: &mut Child, deadline: Deadline) -> Result<Waited, DispatchFailure> {
        iter::repeat_with(Wall::now)
            .take_while(|now| deadline.due(*now) == Due::Pending)
            .find_map(|_| SystemShell::checked(child).transpose())
            .unwrap_or(Ok(Waited::Overdue))
    }

    fn checked(child: &mut Child) -> Result<Option<Waited>, DispatchFailure> {
        let exited = child.try_wait().map_err(DispatchFailure::Collect)?;
        Ok(exited.map(Waited::Exited).or_else(|| {
            thread::sleep(SystemShell::POLL);
            None
        }))
    }

    fn collected(reading: Option<JoinHandle<io::Result<String>>>) -> Result<Printed, DispatchFailure> {
        let text = reading
            .map(|handle| handle.join().map_err(|_panicked| DispatchFailure::Unjoined))
            .transpose()?
            .transpose()
            .map_err(DispatchFailure::Collect)?;
        Ok(Printed::new(text.unwrap_or_default()))
    }

    fn killed(child: &mut Child) -> Result<Ran, DispatchFailure> {
        Command::new(SystemShell::KILL)
            .args(SystemShell::SIGNAL)
            .arg(format!("-{}", child.id()))
            .stderr(Stdio::null())
            .status()
            .and_then(|_signalled| child.wait())
            .map(|_reaped| Ran::Overdue)
            .map_err(DispatchFailure::Kill)
    }
}

impl ShellFiller for SystemShell {
    fn ran(&self, _pass: Pass<'_, shell_ops::Ran>, call: &Call) -> Result<Ran, DispatchFailure> {
        SystemShell::started(call).and_then(SystemShell::finished)
    }

    fn fanned(&self, _pass: Pass<'_, shell_ops::Fanned>, calls: &[Call]) -> Vec<Result<Ran, DispatchFailure>> {
        SystemShell::awaited(calls.iter().map(SystemShell::started).collect())
    }
}
