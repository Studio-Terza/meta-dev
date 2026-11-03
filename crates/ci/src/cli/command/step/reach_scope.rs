mod event;
mod skipped;

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use file_ports::Files;

use super::Step;
use super::branch::Branch;
use super::changed::{Changed, Question};
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports, Text};
use event::Event;
use skipped::Skips;

#[derive(Debug)]
pub struct ReachScope {
    metadata: Journal,
    event: Event,
    base: Branch,
    environment: Journal,
    summary: Journal,
}

impl ReachScope {
    const MAIN: &'static str = "main";
    const WHOLE_WORKSPACE: &'static str = "BUILD_SCOPE=--workspace\n";

    fn recorded(&self, files: &impl Files, report: &Text, out: &mut impl Write) -> Result<(), Failure> {
        let skipped = Skips::from(report.as_ref());
        write!(out, "{report}")?;
        self.environment.append(files, report)?;
        self.summary.append(files, format_args!("{report}{skipped}"))?;
        Ok(write!(out, "{skipped}")?)
    }
}

impl Step for ReachScope {
    fn from_arguments(arguments: &[OsString]) -> Result<ReachScope, Failure> {
        match arguments {
            [metadata, event, base, environment, summary] => Ok(ReachScope {
                metadata: Journal::file(Path::new(metadata)),
                event: Event::from(event.to_str().ok_or(Failure::Usage)?),
                base: Branch::new(base.to_str().ok_or(Failure::Usage)?.to_owned()),
                environment: Journal::file(Path::new(environment)),
                summary: Journal::file(Path::new(summary)),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        Changed::stored_metadata(ports, &self.metadata)?;
        let report = match (self.event, self.base.as_ref() == Self::MAIN) {
            (Event::PullRequest, false) => Changed::answered(Question::Affected, &self.metadata, ports)?,
            (Event::PullRequest, true) => Changed::answered(Question::Widened, &self.metadata, ports)?,
            (Event::Other, _) => Text::new(Self::WHOLE_WORKSPACE.to_owned()),
        };
        self.recorded(ports.files(), &report, out).map(|()| Exit::Success)
    }
}
