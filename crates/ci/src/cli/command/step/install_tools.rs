mod tool;

use std::ffi::{OsStr, OsString};
use std::io;
use std::io::Write;
use std::path::PathBuf;

use file_ports::Files;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports};
use tool::Tool;

#[derive(Debug)]
pub struct InstallTools {
    built: PathBuf,
    destination: PathBuf,
}

impl InstallTools {
    pub(crate) const PROGRAMS: [Tool; 3] = [Tool::REACH, Tool::LINT, Tool::CI];

    fn build_invocation() -> Invocation {
        Invocation::new(
            "cargo",
            ["build", "--locked", "-q"]
                .into_iter()
                .map(OsStr::new)
                .chain(Self::PROGRAMS.iter().flat_map(|tool| [OsStr::new("--bin"), tool.as_ref()])),
        )
    }

    fn copied(&self, files: &impl Files, tool: Tool) -> Result<(), Failure> {
        let from = self.built.join(tool.as_ref());
        files
            .copy(&from, &self.destination.join(tool.as_ref()))
            .map_err(|failure| Failure::Io { path: from, source: io::Error::from(failure) })
    }

    fn installed(&self, files: &impl Files) -> Result<Exit, Failure> {
        files
            .create_directories(&self.destination)
            .map_err(|failure| Failure::Io { path: self.destination.clone(), source: io::Error::from(failure) })?;
        Self::PROGRAMS.into_iter().try_for_each(|tool| self.copied(files, tool)).map(|()| Exit::Success)
    }
}

impl Step for InstallTools {
    fn from_arguments(arguments: &[OsString]) -> Result<InstallTools, Failure> {
        match arguments {
            [built, destination] => {
                Ok(InstallTools { built: PathBuf::from(built), destination: PathBuf::from(destination) })
            }
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure> {
        let build = Self::build_invocation();
        let run = ports.run(&build).map_err(Failure::starting(&build))?;
        run.exit().followed_by(|| self.installed(ports.files()))
    }
}
