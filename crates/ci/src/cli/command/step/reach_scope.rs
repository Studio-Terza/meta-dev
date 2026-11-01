mod skipped;

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use text_lines::Lines;

use super::Step;
use super::journal::Journal;
use crate::cli::command::failure::{Failure, Refusal};
use crate::cli::ports::{Exit, Invocation, Ports};
use skipped::Skipped;

#[derive(Debug)]
pub struct ReachScope {
    metadata: Journal,
    event: String,
    environment: Journal,
    summary: Journal,
}

impl ReachScope {
    const PULL_REQUEST: &'static str = "pull_request";
    const AFFECTED: &'static str = "affected";
    const WHOLE_WORKSPACE: &'static str = "CARGO_SCOPE=--workspace\n";

    fn affected(&self, ports: &impl Ports) -> Result<String, Failure> {
        let changed = Self::asked(
            ports,
            &Invocation::new(
                "git",
                ["-c", "core.quotePath=false", "diff", "--name-only", "--no-renames", "HEAD^1", "HEAD"],
            ),
        )?;
        let arguments = [OsString::from(Self::AFFECTED), self.metadata.as_ref().as_os_str().to_owned()]
            .into_iter()
            .chain(Lines::from(changed.as_str()).map(OsString::from));
        let mut report = Vec::new();
        let mut refusal = Vec::new();
        if reach::Cli::answer(arguments, &mut report, &mut refusal) == ExitCode::SUCCESS {
            Ok(String::from_utf8(report).unwrap_or_default())
        } else {
            Err(Failure::Reach(Refusal::from(refusal)))
        }
    }

    fn asked(ports: &impl Ports, invocation: &Invocation) -> Result<String, Failure> {
        ports.output(invocation).map_err(|source| Failure::Program { program: invocation.program().to_owned(), source })
    }

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
            [metadata, event, environment, summary] => Ok(ReachScope {
                metadata: Journal::file(Path::new(metadata)),
                event: event.to_str().ok_or(Failure::Usage)?.to_owned(),
                environment: Journal::file(Path::new(environment)),
                summary: Journal::file(Path::new(summary)),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let metadata = Self::asked(
            ports,
            &Invocation::new("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"]),
        )?;
        self.metadata.replace(&metadata)?;
        let report =
            if self.event == Self::PULL_REQUEST { self.affected(ports)? } else { Self::WHOLE_WORKSPACE.to_owned() };
        self.recorded(&report, out).map(|()| Exit::Success)
    }
}
