use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use super::Step;
use super::changed::Changed;
use super::install_tools::InstallTools;
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports, Text};

#[derive(Debug)]
pub struct ToolsKey {
    metadata: Journal,
    output: Journal,
}

impl ToolsKey {
    const SOURCES: &'static str = "sources";
    const DELIMITER: &'static str = "TOOLS_SOURCES";

    fn listed(&self) -> Result<Text, Failure> {
        let arguments = [OsString::from(Self::SOURCES), self.metadata.as_ref().as_os_str().to_owned()]
            .into_iter()
            .chain(InstallTools::PROGRAMS.iter().map(|tool| tool.as_ref().to_owned()));
        Changed::reach(arguments)
    }
}

impl Step for ToolsKey {
    fn from_arguments(arguments: &[OsString]) -> Result<ToolsKey, Failure> {
        match arguments {
            [metadata, output] => {
                Ok(ToolsKey { metadata: Journal::file(Path::new(metadata)), output: Journal::file(Path::new(output)) })
            }
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure> {
        Changed::stored_metadata(ports, &self.metadata)?;
        let listed = self.listed()?;
        self.output.append(format_args!("{}<<{}\n{listed}{}\n", Self::SOURCES, Self::DELIMITER, Self::DELIMITER))?;
        Ok(Exit::Success)
    }
}
