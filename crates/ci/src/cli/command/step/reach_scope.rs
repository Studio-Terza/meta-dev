mod skipped;

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use text_lines::Lines;

use super::Step;
use super::changed::Changed;
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports};
use skipped::Skipped;

#[derive(Debug)]
pub struct ReachScope {
    metadata: Journal,
    event: String,
    base: String,
    environment: Journal,
    summary: Journal,
}

impl ReachScope {
    const PULL_REQUEST: &'static str = "pull_request";
    const AFFECTED: &'static str = "affected";
    const WIDENED: &'static str = "widened";
    const MAIN: &'static str = "main";
    const WHOLE_WORKSPACE: &'static str = "CARGO_SCOPE=--workspace\n";

    fn recorded(&self, report: &str, out: &mut impl Write) -> Result<(), Failure> {
        write!(out, "{report}")?;
        self.environment.append(report)?;
        self.summary.append(report)?;
        Lines::from(report).filter_map(|line| Skipped::try_from(line).ok()).try_for_each(|skipped| {
            writeln!(out, "{skipped}")?;
            self.summary.append(format_args!("{skipped}\n"))
        })
    }
}

impl Step for ReachScope {
    fn from_arguments(arguments: &[OsString]) -> Result<ReachScope, Failure> {
        match arguments {
            [metadata, event, base, environment, summary] => Ok(ReachScope {
                metadata: Journal::file(Path::new(metadata)),
                event: event.to_str().ok_or(Failure::Usage)?.to_owned(),
                base: base.to_str().ok_or(Failure::Usage)?.to_owned(),
                environment: Journal::file(Path::new(environment)),
                summary: Journal::file(Path::new(summary)),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let metadata = Changed::asked(
            ports,
            &Invocation::new("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"]),
        )?;
        self.metadata.replace(&metadata)?;
        let report = match (self.event == Self::PULL_REQUEST, self.base == Self::MAIN) {
            (true, false) => Changed::answered(Self::AFFECTED, &self.metadata, ports)?,
            (true, true) => Changed::answered(Self::WIDENED, &self.metadata, ports)?,
            (false, _) => Self::WHOLE_WORKSPACE.to_owned(),
        };
        self.recorded(&report, out).map(|()| Exit::Success)
    }
}
