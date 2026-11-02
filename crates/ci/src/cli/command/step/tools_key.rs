use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use super::Step;
use super::changed::Changed;
use super::install_tools::InstallTools;
use super::journal::Journal;
use crate::cli::command::failure::{Failure, Refusal};
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct ToolsKey {
    metadata: Journal,
    output: Journal,
}

impl ToolsKey {
    const SOURCES: &'static str = "sources";
    const DELIMITER: &'static str = "TOOLS_SOURCES";

    fn listed(&self) -> Result<String, Failure> {
        let arguments = [OsString::from(Self::SOURCES), self.metadata.as_ref().as_os_str().to_owned()]
            .into_iter()
            .chain(InstallTools::PROGRAMS.iter().map(OsString::from));
        let mut report = Vec::new();
        let mut refusal = Vec::new();
        if reach::Cli::answer(arguments, &mut report, &mut refusal) == ExitCode::SUCCESS {
            Ok(String::from_utf8(report).unwrap_or_default())
        } else {
            Err(Failure::Reach(Refusal::from(refusal)))
        }
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
        let metadata = Changed::asked(
            ports,
            &Invocation::new("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"]),
        )?;
        self.metadata.replace(&metadata)?;
        let listed = self.listed()?;
        self.output.append(format_args!("{}<<{}\n{listed}{}\n", Self::SOURCES, Self::DELIMITER, Self::DELIMITER))?;
        Ok(Exit::Success)
    }
}
