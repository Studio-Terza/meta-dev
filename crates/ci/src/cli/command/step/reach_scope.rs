use crate::cli::command::Telling;
use console_ports::Shown;
mod event;
mod skipped;

use std::ffi::OsString;

use file_ports::Files;
use grammar_branch::Base;

use super::Step;
use super::changed::{Changed, Question};
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled, Utf8};
use crate::cli::ports::{Exit, Ports, Text};
use event::Event;
use grammar_output::Record;
use skipped::Skips;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub struct ReachScope {
    #[field(Journaled)]
    metadata: Journal,
    #[field(Utf8, via = |text: String| Event::from(text.as_str()))]
    event: Event,
    #[field(Utf8, via = |text: String| Base::from(text.as_str()))]
    base: Base,
    #[field(Journaled)]
    environment: Journal,
    #[field(Journaled)]
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

    fn recorded(&self, files: &impl Files, report: &Text) -> Result<Vec<Shown>, Failure<'_>> {
        let skipped = Skips::from(report.as_ref());
        self.environment.append(files, report)?;
        self.summary.append(files, format_args!("{report}{skipped}"))?;
        Ok(vec![Shown::said(report), Shown::said(skipped)])
    }
}

impl Step for ReachScope {
    fn from_arguments(given: Vec<OsString>) -> Result<ReachScope, Failure<'static>> {
        ReachScope::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        Changed::stored_metadata(ports, &self.metadata)?;
        let report = self.report(ports)?;
        let said = self.recorded(ports.files(), &report)?;
        Ok(Telling::reported(said, Ok(Exit::Success)))
    }
}
