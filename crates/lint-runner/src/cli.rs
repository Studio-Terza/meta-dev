mod boundary;
mod mode;
mod outcome;
mod refusal;
mod sources;
mod stack_size;
mod workspaces;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

use console_ports::Cargo;
use file_ports::Files;
use lint_reader::SourceFiles;

use mode::Mode;
use outcome::Outcome;
use refusal::Refusal;
use sources::Sources;
use stack_size::StackSize;
use workspaces::Workspaces;

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
        cargo: &impl Cargo,
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> ExitCode {
        match self.mode.workspaces(cargo, &self.roots) {
            Ok(workspaces) => StackSize::LINT
                .ran(|| self.linted(&Sources::new(files, &self.roots, workspaces.as_ref()), out, err))
                .unwrap_or_else(|failure| Self::refused(&failure, err)),
            Err(refused) => Self::refused(&refused, err),
        }
    }

    fn linted(&self, sources: &Sources<impl Files + Sync>, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        thread::scope(|scope| {
            let helpers = self.mode.helpers(scope, sources);
            let read =
                SourceFiles::read_all(sources.files(), sources.roots()).map(|readout| sources.described(readout));
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
