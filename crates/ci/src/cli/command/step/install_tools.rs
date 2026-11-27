use crate::cli::command::Telling;
mod tool;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use file_ports::Files;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Location};
use crate::cli::ports::{Exit, Invocation, Ports};
use grammar_output::Record;
use tool::Tool;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub(crate) struct InstallTools {
    #[field(Location)]
    built: PathBuf,
    #[field(Location)]
    destination: PathBuf,
}

impl InstallTools {
    const PROGRAMS: [Tool; 2] = [Tool::LINT, Tool::CI];

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
            .try_for_each(|(from, outcome)| outcome.map_err(|failure| Failure::unusable(from, failure)))
            .map(|()| Exit::Success)
    }
}

impl Step for InstallTools {
    fn from_arguments(given: Vec<OsString>) -> Result<InstallTools, Failure<'static>> {
        InstallTools::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let build = Self::build_invocation();
        let run = ports.run(&build).map_err(Failure::starting(&build))?;
        Ok(Telling::ended(run.exit().followed_by(|| self.installed(ports.files()))))
    }
}
