use crate::cli::command::Telling;
use std::ffi::OsString;

use super::Step;
use super::changed::Changed;
use super::install_tools::InstallTools;
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled};
use crate::cli::ports::{Exit, Ports, Text};
use grammar_output::Record;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub struct ToolsKey {
    #[field(Journaled)]
    metadata: Journal,
    #[field(Journaled)]
    output: Journal,
}

impl ToolsKey {
    const SOURCES: &'static str = "sources";
    const DELIMITER: &'static str = "TOOLS_SOURCES";

    fn listed(&self, ports: &impl Ports) -> Result<Text, Failure<'static>> {
        let arguments = [OsString::from(Self::SOURCES), self.metadata.as_ref().as_os_str().to_owned()]
            .into_iter()
            .chain(InstallTools::PROGRAMS.iter().map(|tool| tool.as_ref().to_owned()));
        Changed::reach(ports, arguments)
    }
}

impl Step for ToolsKey {
    fn from_arguments(given: Vec<OsString>) -> Result<ToolsKey, Failure<'static>> {
        ToolsKey::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        Changed::stored_metadata(ports, &self.metadata)?;
        let listed = self.listed(ports)?;
        self.output.append(
            ports.files(),
            format_args!("{}<<{}\n{listed}{}\n", Self::SOURCES, Self::DELIMITER, Self::DELIMITER),
        )?;
        Ok(Telling::ended(Ok(Exit::Success)))
    }
}
