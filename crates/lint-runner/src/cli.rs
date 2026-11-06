mod boundary;
mod mode;
mod outcome;
mod sources;
mod stack_size;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

use file_ports::Files;
use lint_layout::Failure;
use lint_reader::{SourceFiles, Stage};

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
        match self.mode.refusal() {
            Some(refused) => Self::refused(refused, err),
            None => StackSize::LINT
                .ran(|| self.linted(SourceFiles::staged(files, &self.roots), files, out, err))
                .unwrap_or_else(|failure| Self::refused(&failure, err)),
        }
    }

    fn linted(
        &self,
        staged: Result<Stage<'_>, Failure>,
        files: &(impl Files + Sync),
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> ExitCode {
        match staged {
            Ok(stage) => self.staged(&Sources::new(files, &stage), out, err),
            Err(failure) => Self::refused(&failure, err),
        }
    }

    fn staged(&self, sources: &Sources<impl Files + Sync>, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        thread::scope(|scope| {
            let helpers = self.mode.helpers(scope, sources);
            let readout = SourceFiles::read_staged(sources.files(), sources.stage());
            let outcome = self.mode.outcome_of(&readout, sources.queue(), helpers, out);
            let written = outcome.violations().map(|violations| Self::written(violations, err));
            written.unwrap_or(ExitCode::SUCCESS)
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
