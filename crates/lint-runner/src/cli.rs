mod boundary;
mod mode;
mod outcome;
mod sources;
mod stack_size;
mod verdict;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

use cargo_ports::Cargo;
use file_ports::Files;
use lint_layout::Failure;
use lint_reader::{SourceFiles, Stage};

use mode::Mode;
use outcome::Outcome;
use sources::{Ports, Sources};
use stack_size::StackSize;
pub use verdict::Verdict;

#[derive(Debug)]
pub struct Cli {
    roots: Vec<PathBuf>,
    mode: Mode,
}

impl Cli {
    pub fn from_args(arguments: impl Iterator<Item = OsString>) -> Cli {
        let given: Vec<OsString> = arguments.collect();
        let mode = Mode::of_arguments(&given);
        let named = mode.roots(&given);
        let roots = named.first().map(|_| ()).map_or_else(|| vec![PathBuf::from(".")], |()| named);
        Cli { roots, mode }
    }

    #[must_use]
    pub fn report_to(
        &self,
        files: &(impl Files + Sync),
        cargo: &(impl Cargo + Sync),
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> ExitCode {
        self.verdict(files, cargo, out, err).into()
    }

    #[must_use]
    pub fn verdict(
        &self,
        files: &(impl Files + Sync),
        cargo: &(impl Cargo + Sync),
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> Verdict {
        match self.mode.refusal() {
            Some(refused) => Self::refused(refused, err),
            None => StackSize::LINT
                .ran(|| self.linted(SourceFiles::staged(files, &self.roots), Ports::new(files, cargo), out, err))
                .unwrap_or_else(|failure| Self::refused(&failure, err)),
        }
    }

    fn linted(
        &self,
        staged: Result<Stage<'_>, Failure>,
        ports: Ports<'_, impl Files + Sync, impl Cargo + Sync>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        match staged {
            Ok(stage) => self.staged(&Sources::new(ports, &stage), out, err),
            Err(failure) => Self::refused(&failure, err),
        }
    }

    fn staged(
        &self,
        sources: &Sources<impl Files + Sync, impl Cargo + Sync>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        thread::scope(|scope| {
            let helpers = self.mode.helpers(scope, sources);
            let readout = SourceFiles::read_staged(sources.ports().files(), sources.stage());
            let outcome = self.mode.outcome_of(&readout, sources.queue(), helpers, out);
            let written = outcome.violations().map(|violations| Self::written(violations, err));
            written.unwrap_or(Verdict::Clean)
        })
    }

    fn refused(failure: &impl fmt::Display, err: &mut impl Write) -> Verdict {
        writeln!(err, "{failure}").ok();
        Verdict::Refused
    }

    fn written(violations: &Outcome<'_>, err: &mut impl Write) -> Verdict {
        write!(err, "{violations}").ok();
        Verdict::Violations
    }
}
