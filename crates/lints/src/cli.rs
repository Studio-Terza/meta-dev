use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use lint_core::Failure;

use crate::{Lints, Outcome};

#[derive(Debug)]
pub struct Cli {
    roots: Vec<PathBuf>,
}

impl Cli {
    pub fn from_args(arguments: impl Iterator<Item = OsString>) -> Cli {
        let mut roots: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
        if roots.is_empty() {
            roots.push(PathBuf::from("."));
        }
        Cli { roots }
    }

    pub fn outcome(&self) -> Result<Outcome, Failure> {
        Lints::check_all(&self.roots)
    }

    #[must_use]
    pub fn run(&self) -> ExitCode {
        match self.outcome() {
            Ok(Outcome::Clean) => ExitCode::SUCCESS,
            Ok(violations) => {
                eprint!("{violations}");
                ExitCode::FAILURE
            }
            Err(failure) => {
                eprintln!("{failure}");
                ExitCode::FAILURE
            }
        }
    }
}
