use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use lint_bool_methods::BoolMethods;
use lint_core::Lint;
use lint_paired_apis::PairedApis;
use lint_reader::SourceFiles;
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
        self.report_to(&mut io::stdout(), &mut io::stderr())
    }

    #[must_use]
    pub fn report_to(&self, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        match SourceFiles::read_all(&self.roots).map(|readout| {
            let files = readout.files();
            write!(out, "{}{}{}", TestBudget::measured(files), BoolMethods::report(files), PairedApis::report(files))
                .ok();
            Lints::checked(readout)
        }) {
            Ok(Outcome::Clean) => ExitCode::SUCCESS,
            Ok(violations) => {
                write!(err, "{violations}").ok();
                ExitCode::FAILURE
            }
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }
}
