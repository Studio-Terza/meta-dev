use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use text_lines::Lines;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct ReachScope {
    metadata: PathBuf,
    event: String,
}

impl ReachScope {
    const PULL_REQUEST: &'static str = "pull_request";
    const AFFECTED: &'static str = "affected";
    const WHOLE_WORKSPACE: &'static str = "CARGO_SCOPE=--workspace";

    fn affected(&self, ports: &impl Ports, out: &mut impl Write) -> Result<(), Failure> {
        let changed =
            Self::asked(ports, &Invocation::new("git", ["diff", "--name-only", "--no-renames", "HEAD^1", "HEAD"]))?;
        let arguments = [OsString::from(Self::AFFECTED), self.metadata.clone().into_os_string()]
            .into_iter()
            .chain(Lines::from(changed.as_str()).map(OsString::from));
        let mut report = Vec::new();
        let mut refusal = Vec::new();
        if reach::Cli::answer(arguments, &mut report, &mut refusal) == ExitCode::SUCCESS {
            Ok(out.write_all(&report)?)
        } else {
            Err(Failure::Reach(String::from_utf8(refusal).unwrap_or_default()))
        }
    }

    fn asked(ports: &impl Ports, invocation: &Invocation) -> Result<String, Failure> {
        ports.output(invocation).map_err(|source| Failure::Program { program: invocation.program().to_owned(), source })
    }
}

impl Step for ReachScope {
    fn from_arguments(arguments: &[OsString]) -> Result<ReachScope, Failure> {
        match arguments {
            [metadata, event] => Ok(ReachScope {
                metadata: PathBuf::from(metadata),
                event: event.to_str().ok_or(Failure::Usage)?.to_owned(),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let metadata =
            Self::asked(ports, &Invocation::new("cargo", ["metadata", "--no-deps", "--format-version", "1"]))?;
        fs::write(&self.metadata, metadata).map_err(|source| Failure::Io { path: self.metadata.clone(), source })?;
        let scoped = if self.event == Self::PULL_REQUEST {
            self.affected(ports, out)
        } else {
            Ok(writeln!(out, "{}", Self::WHOLE_WORKSPACE)?)
        };
        scoped.map(|()| Exit::Success)
    }
}
