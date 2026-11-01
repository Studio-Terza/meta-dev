use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Invocation, Ports};

#[derive(Debug)]
pub struct InstallTools {
    built: PathBuf,
    destination: PathBuf,
}

impl InstallTools {
    const PROGRAMS: [&'static str; 3] = ["reach", "lint", "ci"];

    fn build_invocation() -> Invocation {
        Invocation::new(
            "cargo",
            ["build", "--locked", "-q"].into_iter().chain(Self::PROGRAMS.iter().flat_map(|name| ["--bin", *name])),
        )
    }

    fn copied(&self, name: &str) -> Result<(), Failure> {
        let from = self.built.join(name);
        fs::copy(&from, self.destination.join(name)).map(drop).map_err(|source| Failure::Io { path: from, source })
    }

    fn installed(&self) -> Result<Exit, Failure> {
        fs::create_dir_all(&self.destination)
            .map_err(|source| Failure::Io { path: self.destination.clone(), source })?;
        Self::PROGRAMS.iter().try_for_each(|name| self.copied(name)).map(|()| Exit::Success)
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
        run.exit().followed_by(|| self.installed())
    }
}
