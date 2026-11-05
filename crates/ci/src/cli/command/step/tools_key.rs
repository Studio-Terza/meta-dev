use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use super::Step;
use super::changed::Changed;
use super::install_tools::InstallTools;
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports, Text};

#[derive(Debug)]
pub struct ToolsKey {
    metadata: Journal,
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
    fn from_arguments(arguments: Vec<OsString>) -> Result<ToolsKey, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let metadata = &arguments.taken()?;
        let output = &arguments.taken()?;
        arguments.ended()?;
        Ok(ToolsKey { metadata: Journal::file(Path::new(metadata)), output: Journal::file(Path::new(output)) })
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        Changed::stored_metadata(ports, &self.metadata)?;
        let listed = self.listed(ports)?;
        self.output.append(
            ports.files(),
            format_args!("{}<<{}\n{listed}{}\n", Self::SOURCES, Self::DELIMITER, Self::DELIMITER),
        )?;
        Ok(Exit::Succeeded)
    }
}
