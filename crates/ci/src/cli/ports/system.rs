mod tee;

use std::fs::{self, File};
use std::io;
use std::num::NonZeroU8;
use std::path::Path;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::time::Instant;

use file_ports::Disk;

use super::{Exit, Invocation, Ports, Run, Text};
use crate::cli::ignored::Ignored;
use invariants::Ignoring;
pub(crate) use tee::Tee;

#[derive(Debug)]
pub struct System;

impl System {
    fn exit_of(status: ExitStatus) -> Exit {
        Some(status).filter(ExitStatus::success).map_or_else(
            || {
                Exit::Failure(
                    status
                        .code()
                        .and_then(|code| u8::try_from(code).ignoring(Ignored::BeyondRange))
                        .and_then(NonZeroU8::new)
                        .unwrap_or(NonZeroU8::MIN),
                )
            },
            |_| Exit::Success,
        )
    }

    fn printed(Output { status, stdout, stderr }: Output) -> io::Result<Text> {
        match Some(status).filter(ExitStatus::success).map(|_| stdout).ok_or(stderr) {
            Ok(printed) => String::from_utf8(printed).map(Text::new).map_err(io::Error::other),
            Err(failed) => {
                Err(io::Error::other(String::from_utf8(failed).ignoring(Ignored::NotUtf8).unwrap_or_default()))
            }
        }
    }

    fn command(invocation: &Invocation) -> Command {
        let mut command = Command::new(invocation.program());
        command.args(invocation.arguments());
        command
    }
}

impl Ports for System {
    type Files = Disk;

    fn files(&self) -> &Disk {
        &Disk
    }

    fn run(&self, invocation: &Invocation) -> io::Result<Run> {
        let started = Instant::now();
        let status = Self::command(invocation).status()?;
        Ok(Run::new(Self::exit_of(status), started.elapsed()))
    }

    fn output(&self, invocation: &Invocation) -> io::Result<Text> {
        Self::command(invocation).output().and_then(Self::printed)
    }

    fn logged(&self, invocation: &Invocation, log: &Path) -> io::Result<Run> {
        let started = Instant::now();
        log.parent().map_or(Ok(()), fs::create_dir_all)?;
        let mut tee = Tee::new(io::stdout(), File::create(log)?);
        let mut child = Self::command(invocation).stdout(Stdio::piped()).spawn()?;
        let copied = child.stdout.take().map_or(Ok(0), |mut stdout| io::copy(&mut stdout, &mut tee));
        let status = child.wait()?;
        copied?;
        Ok(Run::new(Self::exit_of(status), started.elapsed()))
    }

    fn runs(&self, invocations: &[Invocation]) -> Vec<io::Result<Run>> {
        invocations.iter().map(|invocation| self.run(invocation)).collect()
    }

    fn outputs(&self, invocations: &[Invocation]) -> Vec<io::Result<Text>> {
        invocations.iter().map(|invocation| self.output(invocation)).collect()
    }
}
