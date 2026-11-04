mod boundary;
mod mode;
mod outcome;
mod sources;
mod stack_size;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::panic::resume_unwind;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread::{self, ScopedJoinHandle};

use file_ports::Files;
use lint_reader::SourceFiles;

use mode::Mode;
use outcome::Outcome;
use sources::Sources;
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

    fn reported_to(&self, files: &(impl Files + Sync), out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        match self.mode.refusal() {
            Some(refused) => Self::refused(refused, err),
            None => self.ended(&Sources::new(files, &self.roots), out, err),
        }
    }

    fn ended(&self, sources: &Sources<impl Files + Sync>, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        thread::scope(|scope| {
            let helpers = self.mode.helpers(scope, sources);
            let read = SourceFiles::read_all(sources.files(), sources.roots());
            let outcome = read.as_ref().map(|readout| self.mode.outcome_of(readout, helpers, out));
            let refusal = outcome.as_ref().err().map(|failure| Self::refused(failure, err));
            let written =
                outcome.ok().as_ref().and_then(Outcome::violations).map(|violations| Self::written(violations, err));
            refusal.or(written).unwrap_or(ExitCode::SUCCESS)
        })
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
