mod boundary;
mod mode;
mod outcome;
mod sources;
mod stack_size;
mod timings;
mod verdict;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

use cargo_ports::{Cargo, CargoFiller};
use file_ports::Files;
use lint_layout::Failure;
use lint_reader::{SourceFiles, Stage};
use port_core::Run;
use timer_ports::{Timer, TimerFiller};

use mode::Mode;
use outcome::Outcome;
use sources::{Sources, Wired};
use stack_size::StackSize;
use timings::{Measured, Step, Stopwatch, Timings, Worker};
pub use verdict::Verdict;

#[derive(Debug)]
pub struct Cli {
    roots: Vec<PathBuf>,
    mode: Mode,
    timings: Timings,
}

#[derive(Debug)]
pub struct Ports<'files, F, C, T> {
    files: &'files F,
    cargo: C,
    timer: T,
}

impl<'files, F: Files + Sync, C: CargoFiller + Sync, T: TimerFiller + Sync> Ports<'files, F, C, T> {
    pub fn new(files: &'files F, cargo: C, timer: T) -> Ports<'files, F, C, T> {
        Ports { files, cargo, timer }
    }
}

impl Cli {
    pub fn from_args(arguments: impl Iterator<Item = OsString>) -> Cli {
        let flagged: Vec<OsString> = arguments.collect();
        let timings = Timings::of_arguments(&flagged);
        let given = Timings::unflagged(flagged);
        let mode = Mode::of_arguments(&given);
        let named = mode.roots(&given);
        let roots = named.first().map(|_| ()).map_or_else(|| vec![PathBuf::from(".")], |()| named);
        Cli { roots, mode, timings }
    }

    #[must_use]
    pub fn report_to(
        &self,
        ports: Ports<'_, impl Files + Sync, impl CargoFiller + Sync, impl TimerFiller + Sync>,
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> ExitCode {
        self.verdict(ports, out, err).into()
    }

    #[must_use]
    pub fn verdict(
        &self,
        ports: Ports<'_, impl Files + Sync, impl CargoFiller + Sync, impl TimerFiller + Sync>,
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> Verdict {
        let run = Run::root();
        let cargo = run.port(ports.cargo);
        let timer = run.port(ports.timer);
        self.metered(&Wired::new(ports.files, &cargo, &timer, self.timings), out, err)
    }

    fn metered(
        &self,
        wired: &Wired<'_, impl Files + Sync, impl Cargo + Sync, impl Timer + Sync>,
        out: &mut (impl Write + Send),
        err: &mut (impl Write + Send),
    ) -> Verdict {
        match self.mode.refusal() {
            Some(refused) => Verdict::refused(refused, err),
            None => StackSize::LINT
                .ran(|| self.linted(self.staged(wired), wired, out, err))
                .unwrap_or_else(|failure| Verdict::refused(&failure, err)),
        }
    }

    fn staged(
        &self,
        wired: &Wired<'_, impl Files + Sync, impl Cargo, impl Timer>,
    ) -> Result<Measured<Stage<'_>>, Failure> {
        Stopwatch::measured(wired.timer(), Step::Stage, || SourceFiles::staged(wired.files(), &self.roots)).transposed()
    }

    fn linted(
        &self,
        staged: Result<Measured<Stage<'_>>, Failure>,
        wired: &Wired<'_, impl Files + Sync, impl Cargo + Sync, impl Timer + Sync>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        match staged {
            Ok(stage) => self.sourced(&Sources::new(wired, &stage), out, err),
            Err(failure) => Verdict::refused(&failure, err),
        }
    }

    fn sourced(
        &self,
        sources: &Sources<impl Files + Sync, impl Cargo + Sync, impl Timer + Sync>,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        thread::scope(|scope| {
            let run = self.mode.started(scope, sources);
            run.verdict_of(&sources.read(Worker::MAIN), sources, out, err)
        })
    }
}
