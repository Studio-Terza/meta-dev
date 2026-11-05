mod tool;

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::PathBuf;

use file_ports::Files;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
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

    fn installed(&self, files: &impl Files) -> Result<Exit, Failure<'_>> {
        files.create_directories(&self.destination).map_err(|failure| Failure::unusable(&self.destination, failure))?;
        let built: Vec<PathBuf> = Self::PROGRAMS.iter().map(|tool| self.built.join(tool.as_ref())).collect();
        let copied = files.copy_each_into(&built, &self.destination);
        built
            .into_iter()
            .zip(copied)
            .try_for_each(|(from, copied)| copied.map_err(|failure| Failure::unusable(from, failure)))
            .map(|()| Exit::Succeeded)
    }
}

impl Step for InstallTools {
    fn from_arguments(arguments: Vec<OsString>) -> Result<InstallTools, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let built = &arguments.taken()?;
        let destination = &arguments.taken()?;
        arguments.ended()?;
        Ok(InstallTools { built: PathBuf::from(built), destination: PathBuf::from(destination) })
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let build = Self::build_invocation();
        let run = ports.run(&build).map_err(Failure::starting(&build))?;
        run.exit().followed_by(|| self.installed(ports.files()))
    }
}
