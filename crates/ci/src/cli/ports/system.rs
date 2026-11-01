use std::io;
use std::num::NonZeroU8;
use std::process::{Command, ExitStatus};
use std::time::Instant;

use super::{Exit, Invocation, Ports, Run};

#[derive(Debug)]
pub struct System;

impl System {
    fn exit_of(status: ExitStatus) -> Exit {
        if status.success() {
            Exit::Success
        } else {
            Exit::Failure(
                status
                    .code()
                    .and_then(|code| u8::try_from(code).ok())
                    .and_then(NonZeroU8::new)
                    .unwrap_or(NonZeroU8::MIN),
            )
        }
    }

    fn command(invocation: &Invocation) -> Command {
        let mut command = Command::new(invocation.program());
        command.args(invocation.arguments());
        command
    }
}

impl Ports for System {
    fn run(&self, invocation: &Invocation) -> io::Result<Run> {
        let started = Instant::now();
        let status = Self::command(invocation).status()?;
        Ok(Run::new(Self::exit_of(status), started.elapsed()))
    }

    fn output(&self, invocation: &Invocation) -> io::Result<String> {
        let output = Self::command(invocation).output()?;
        if output.status.success() {
            String::from_utf8(output.stdout).map_err(io::Error::other)
        } else {
            Err(io::Error::other(String::from_utf8(output.stderr).unwrap_or_default()))
        }
    }
}
