use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports, Text};

#[derive(Debug)]
pub(crate) struct Changed;

impl Changed {
    pub(crate) fn asked(ports: &impl Ports, invocation: &Invocation) -> Result<Text, Failure<'static>> {
        ports.output(invocation).map_err(|source| Failure::Program { program: invocation.program().to_owned(), source })
    }

    pub(crate) fn workspace_metadata(ports: &impl Ports) -> Result<Text, Failure<'static>> {
        Self::asked(ports, &Invocation::new("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"]))
    }
}
