mod boundary;
mod mode;
mod outcome;
mod sources;
mod stack_size;

use std::ffi::OsString;
use std::io::Write;
use std::panic::resume_unwind;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread::{self, ScopedJoinHandle};

use file_ports::Files;
use lint_layout::Failure;
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
        Cli { roots: if roots.is_empty() { vec![PathBuf::from(".")] } else { roots }, mode }
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
        match spawned {
            Ok(Ok(exit)) => exit,
            Ok(Err(payload)) => resume_unwind(payload),
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }

    fn reported_to(&self, files: &(impl Files + Sync), out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        match self.mode.refusal() {
            Some(refused) => {
                writeln!(err, "{refused}").ok();
                ExitCode::FAILURE
            }
            None => self.ended(&Sources::new(files, &self.roots), out, err),
        }
    }

    fn ended(&self, sources: &Sources<impl Files + Sync>, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        thread::scope(|scope| {
            let helpers = self.mode.helpers(scope, sources);
            Self::exited(
                SourceFiles::read_all(sources.files(), sources.roots())
                    .as_ref()
                    .map(|readout| self.mode.outcome_of(readout, helpers, out)),
                err,
            )
        })
    }

    fn exited(outcome: Result<Outcome<'_>, &Failure>, err: &mut impl Write) -> ExitCode {
        match outcome {
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
