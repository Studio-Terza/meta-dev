mod mode;
mod outcome;
mod stack_size;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::panic::resume_unwind;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread::{self, ScopedJoinHandle};

use file_ports::Files;
use lint_core::Readout;
use lint_layout::Failure;
use lint_reader::SourceFiles;

use mode::Mode;
use outcome::Outcome;
use stack_size::StackSize;

#[derive(Debug)]
pub struct Cli {
    roots: Vec<PathBuf>,
    mode: Mode,
}

impl Cli {
    pub fn from_args(arguments: impl Iterator<Item = OsString>) -> Cli {
        let given: Vec<OsString> = arguments.collect();
        let mode = Mode::of_arguments(&given);
        let roots = mode.roots(&given);
        let roots = roots.first().map(|_| ()).map_or_else(|| vec![PathBuf::from(".")], |()| roots);
        Cli { roots, mode }
    }

    #[must_use]
    pub fn report_to(
        &self,
        files: &(impl Files + Sync),
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> ExitCode {
        let spawned = thread::scope(|scope| {
            StackSize::LINT
                .builder()
                .spawn_scoped(scope, || self.reported_to(files, out, err))
                .map(ScopedJoinHandle::join)
        });
        spawned.map_or_else(
            |failure| Self::refused(&failure, err),
            |joined| joined.unwrap_or_else(|payload| resume_unwind(payload)),
        )
    }

    fn reported_to(&self, files: &impl Files, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        match self.mode.refusal() {
            Some(refused) => Self::refused(refused, err),
            None => self.ended(SourceFiles::read_all(files, &self.roots).as_ref(), out, err),
        }
    }

    fn ended(&self, read: Result<&Readout, &Failure>, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        match read.map(|readout| self.mode.outcome_of(readout, out)) {
            Ok(outcome) => outcome.violations().map_or(ExitCode::SUCCESS, |violations| Self::written(violations, err)),
            Err(failure) => Self::refused(failure, err),
        }
    }

    fn refused(failure: &impl fmt::Display, err: &mut impl Write) -> ExitCode {
        writeln!(err, "{failure}").ok();
        ExitCode::FAILURE
    }

    fn written(violations: &Outcome<'_>, err: &mut impl Write) -> ExitCode {
        write!(err, "{violations}").ok();
        ExitCode::FAILURE
    }
}
