mod event;
mod skipped;

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use file_ports::Files;
use grammar::branch::Base;

use super::Step;
use super::changed::{Changed, Question};
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports, Text};
use event::Event;
use skipped::Skips;

#[derive(Debug)]
pub struct ReachScope {
    metadata: Journal,
    event: Event,
    base: Base,
    environment: Journal,
    summary: Journal,
}

impl ReachScope {
    const WHOLE_WORKSPACE: &'static str = "BUILD_SCOPE=--workspace\n";

    fn report(&self, ports: &impl Ports) -> Result<Text, Failure<'_>> {
        match self.event {
            Event::PullRequest => Changed::answered(self.question(), &self.metadata, ports),
            Event::Other => Ok(Text::new(Self::WHOLE_WORKSPACE.to_owned())),
        }
    }

    fn question(&self) -> Question {
        match self.base {
            Base::Main | Base::Project => Question::Widened,
            Base::Other => Question::Affected,
        }
    }

    fn recorded(&self, files: &impl Files, report: &Text, out: &mut impl Write) -> Result<(), Failure<'_>> {
        let skipped = Skips::from(report.as_ref());
        write!(out, "{report}")?;
        self.environment.append(files, report)?;
        self.summary.append(files, format_args!("{report}{skipped}"))?;
        Ok(write!(out, "{skipped}")?)
    }
}

impl Step for ReachScope {
    fn from_arguments(given: Vec<OsString>) -> Result<ReachScope, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let metadata = &arguments.taken()?;
        let event = &arguments.taken()?;
        let base = &arguments.taken()?;
        let environment = &arguments.taken()?;
        let summary = &arguments.taken()?;
        arguments.ended()?;
        Ok(ReachScope {
            metadata: Journal::file(Path::new(metadata)),
            event: Event::from(event.to_str().ok_or(Failure::Usage)?),
            base: Base::from(base.to_str().ok_or(Failure::Usage)?),
            environment: Journal::file(Path::new(environment)),
            summary: Journal::file(Path::new(summary)),
        })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        Changed::stored_metadata(ports, &self.metadata)?;
        let report = self.report(ports)?;
        self.recorded(ports.files(), &report, out).map(|()| Exit::Success)
    }
}
