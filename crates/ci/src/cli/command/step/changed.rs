use std::ffi::OsString;
use std::process::ExitCode;

use text_lines::Lines;

use super::journal::Journal;
use crate::cli::command::failure::{Failure, Refusal};
use crate::cli::ports::{Invocation, Ports};

#[derive(Debug)]
pub(crate) struct Changed;

impl Changed {
    const AFFECTED: &'static str = "affected";

    pub(crate) fn asked(ports: &impl Ports, invocation: &Invocation) -> Result<String, Failure> {
        ports.output(invocation).map_err(|source| Failure::Program { program: invocation.program().to_owned(), source })
    }

    pub(crate) fn answered(command: &str, metadata: &Journal, ports: &impl Ports) -> Result<String, Failure> {
        let changed = Self::asked(
            ports,
            &Invocation::new(
                "git",
                ["-c", "core.quotePath=false", "diff", "--name-only", "--no-renames", "HEAD^1", "HEAD"],
            ),
        )?;
        let arguments = [OsString::from(command), metadata.as_ref().as_os_str().to_owned()]
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

    pub(crate) fn affected(metadata: &Journal, ports: &impl Ports) -> Result<String, Failure> {
        Self::answered(Self::AFFECTED, metadata, ports)
    }
}
