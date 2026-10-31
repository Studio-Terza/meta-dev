use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::{Lints, Outcome};

#[derive(Debug)]
pub struct Cli {
    root: PathBuf,
}

impl Cli {
    pub fn from_args(mut arguments: impl Iterator<Item = OsString>) -> Cli {
        Cli { root: arguments.next().map_or_else(|| PathBuf::from("."), PathBuf::from) }
    }

    #[must_use]
    pub fn run(&self) -> ExitCode {
        match Lints::check(&self.root) {
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
