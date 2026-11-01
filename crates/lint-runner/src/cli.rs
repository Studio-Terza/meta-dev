use std::ffi::OsString;
use std::io::{self, Write};
use std::panic::resume_unwind;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread::{self, ScopedJoinHandle};

use lint_bool_methods::BoolMethods;
use lint_core::{Lint, Readout};
use lint_paired_apis::PairedApis;
use lint_reader::SourceFiles;
use lint_test_budget::TestBudget;

use crate::{Elegance, Lints, Mode, Outcome, StackSize};

#[derive(Debug)]
pub struct Cli {
    roots: Vec<PathBuf>,
    mode: Mode,
}

impl Cli {
    pub fn from_args(arguments: impl Iterator<Item = OsString>) -> Cli {
        let given: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
        let roots: Vec<PathBuf> =
            given.iter().filter(|argument| argument.as_os_str() != Mode::REPORT_FLAG).cloned().collect();
        Cli { roots: if roots.is_empty() { vec![PathBuf::from(".")] } else { roots }, mode: Mode::of_arguments(&given) }
    }

    #[must_use]
    pub fn run(&self) -> ExitCode {
        self.report_to(&mut io::stdout(), &mut io::stderr())
    }

    #[must_use]
    pub fn report_to(&self, out: &mut (impl Write + Send), err: &mut (impl Write + Send)) -> ExitCode {
        let spawned = thread::scope(|scope| {
            thread::Builder::new()
                .stack_size(StackSize::LINT.bytes())
                .spawn_scoped(scope, || self.reported_to(out, err))
                .map(ScopedJoinHandle::join)
        });
        match spawned {
            Ok(Ok(exit)) => exit,
            Ok(Err(payload)) => resume_unwind(payload),
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }

    fn reported_to(&self, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        match SourceFiles::read_all(&self.roots).map(|readout| self.outcome_of(readout, out)) {
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

    fn outcome_of(&self, readout: Readout, out: &mut impl Write) -> Outcome {
        let files = readout.files();
        match self.mode {
            Mode::Gate => {
                write!(
                    out,
                    "{}{}{}",
                    TestBudget::measured(files),
                    BoolMethods::report(files),
                    PairedApis::report(files)
                )
                .ok();
                Lints::checked(readout)
            }
            Mode::Report => {
                Elegance::reports(files).iter().try_for_each(|report| write!(out, "{report}")).ok();
                Outcome::Clean
            }
        }
    }
}
