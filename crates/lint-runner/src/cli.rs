use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use lint_core::SourceFiles;
use lint_test_budget::TestBudget;

use crate::{Lints, Outcome};

#[derive(Debug)]
pub struct Cli {
    roots: Vec<PathBuf>,
}

impl Cli {
    pub fn from_args(arguments: impl Iterator<Item = OsString>) -> Cli {
        let given: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
        Cli { roots: if given.is_empty() { vec![PathBuf::from(".")] } else { given } }
    }

    #[must_use]
    pub fn run(&self) -> ExitCode {
        match SourceFiles::read_all(&self.roots).map(|readout| {
            print!("{}", TestBudget::measured(readout.files()));
            Lints::checked(readout)
        }) {
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
