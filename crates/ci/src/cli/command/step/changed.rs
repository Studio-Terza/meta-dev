mod question;

use std::ffi::OsString;

use text_lines::Lines;

use super::journal::Journal;
use crate::cli::command::failure::{Failure, Refusal};
use crate::cli::ports::{Invocation, Ports, Text};
pub(crate) use question::Question;

#[derive(Debug)]
pub(crate) struct Changed;

impl Changed {
    pub(crate) fn asked(ports: &impl Ports, invocation: &Invocation) -> Result<Text, Failure> {
        ports.output(invocation).map_err(|source| Failure::Program { program: invocation.program().to_owned(), source })
    }

    pub(crate) fn stored_metadata(ports: &impl Ports, metadata: &Journal) -> Result<(), Failure> {
        let text = Self::asked(
            ports,
            &Invocation::new("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"]),
        )?;
        metadata.replace(ports.files(), &text)
    }

    pub(crate) fn reach(ports: &impl Ports, arguments: impl Iterator<Item = OsString>) -> Result<Text, Failure> {
        ports.reached(&arguments.collect::<Vec<_>>()).map_err(Self::refused)
    }

    pub(crate) fn reaches(ports: &impl Ports, questions: &[Vec<OsString>]) -> Vec<Result<Text, Failure>> {
        ports.reaches(questions).into_iter().map(|answer| answer.map_err(Self::refused)).collect()
    }

    fn refused(refusal: Text) -> Failure {
        Failure::Reach(Refusal::from(refusal))
    }

    pub(crate) fn answered(question: Question, metadata: &Journal, ports: &impl Ports) -> Result<Text, Failure> {
        let changed = Self::asked(
            ports,
            &Invocation::new(
                "git",
                ["-c", "core.quotePath=false", "diff", "--name-only", "--no-renames", "HEAD^1", "HEAD"],
            ),
        )?;
        let arguments = [OsString::from(question), metadata.as_ref().as_os_str().to_owned()]
            .into_iter()
            .chain(Lines::from(changed.as_ref()).map(OsString::from));
        Self::reach(ports, arguments)
    }
}
