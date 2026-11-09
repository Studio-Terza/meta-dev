mod boundary;
mod mode;
mod outcome;
mod refusal;
mod sources;
mod stack_size;
mod verdict;
mod workspaces;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

use cargo_ports::{Cargo, CargoFiller};
use file_ports::Files;
use lint_layout::Failure;
use lint_reader::{SourceFiles, Stage};
use port_core::Run;

use mode::Mode;
use outcome::Outcome;
use refusal::Refusal;
use sources::{Ports, Sources, Surveyed};
use stack_size::StackSize;
pub use verdict::Verdict;
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
        let named = mode.roots(&given);
        let roots = named.first().map(|_| ()).map_or_else(|| vec![PathBuf::from(".")], |()| named);
        Cli { roots, mode }
    }

    #[must_use]
    pub fn report_to(
        &self,
        files: &(impl Files + Sync),
        cargo: impl CargoFiller + Sync,
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> ExitCode {
        self.verdict(files, cargo, out, err).into()
    }

    #[must_use]
    pub fn verdict(
        &self,
        files: &(impl Files + Sync),
        cargo: impl CargoFiller + Sync,
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> Verdict {
        let run = Run::root();
        let port = run.port(cargo);
        let surveyed = self
            .mode
            .workspaces(&port, &self.roots)
            .map(|workspaces| Surveyed::new(Ports::new(files, &port), workspaces));
        self.judged(surveyed, out, err)
    }

    fn judged(
        &self,
        surveyed: Result<Surveyed<'_, impl Files + Sync, impl Cargo + Sync>, Refusal<'_>>,
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> Verdict {
        match surveyed {
            Ok(surveyed) => StackSize::LINT
                .ran(|| self.linted(SourceFiles::staged(surveyed.ports().files(), &self.roots), surveyed, out, err))
                .unwrap_or_else(|failure| Self::refused(&failure, err)),
            Err(refused) => Self::refused(&refused, err),
        }
    }

    fn linted(
        &self,
        staged: Result<Stage<'_>, Failure>,
        surveyed: Surveyed<'_, impl Files + Sync, impl Cargo + Sync>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        match staged {
            Ok(stage) => self.staged(&Sources::new(surveyed, &stage), out, err),
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
            let run = self.mode.started(scope, sources);
            let surveyed = sources.surveyed();
            let readout = surveyed.described(SourceFiles::read_staged(surveyed.ports().files(), sources.stage()));
            run.outcome_of(&readout, sources.queue(), out).written(err)
        })
    }

    fn refused(failure: &impl fmt::Display, err: &mut impl Write) -> Verdict {
        writeln!(err, "{failure}").ok();
        Verdict::Refused
    }
}
