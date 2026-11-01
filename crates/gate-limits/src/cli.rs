use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::failure::Failure;
use crate::gate::Gate;

#[derive(Debug)]
pub struct Cli {
    limits: PathBuf,
    measured: PathBuf,
}

impl Cli {
    const DEFAULT_LIMITS: &'static str = "ci/limits.toml";
    const DEFAULT_MEASURED: &'static str = "ci/measured.toml";

    pub fn from_args(mut arguments: impl Iterator<Item = OsString>) -> Cli {
        let limits = arguments.next().map_or_else(|| PathBuf::from(Self::DEFAULT_LIMITS), PathBuf::from);
        let measured = arguments.next().map_or_else(|| PathBuf::from(Self::DEFAULT_MEASURED), PathBuf::from);
        Cli { limits, measured }
    }

    #[must_use]
    pub fn run(&self) -> ExitCode {
        match self.gate() {
            Ok(gate) => {
                print!("{gate}");
                if gate.blocks() { ExitCode::FAILURE } else { ExitCode::SUCCESS }
            }
            Err(failure) => {
                eprintln!("{failure}");
                ExitCode::FAILURE
            }
        }
    }

    fn gate(&self) -> Result<Gate, Failure> {
        Gate::judge(&Self::read(&self.limits)?, &Self::read(&self.measured)?)
    }

    fn read(path: &Path) -> Result<String, Failure> {
        fs::read_to_string(path).map_err(|source| Failure::Io { path: path.to_path_buf(), source })
    }
}
