use std::sync::OnceLock;

use cargo_ports::Cargo;
use code_model::Model;
use code_reader::Reader;
use file_ports::Files;
use lint_core::Readout;
use lint_reader::{CrateSources, Described, SourceFiles, Stage, Unlocated};
use timer_ports::Timer;

use super::boundary::Queue;
use super::timings::{Measured, Step, Stopwatch, Table, Timing, Timings, Worker};

#[derive(Debug)]
pub(crate) struct Wired<'run, F, C, T> {
    files: &'run F,
    cargo: &'run C,
    timer: &'run T,
    total: Stopwatch<'run, T>,
    shown: Timings,
}

#[derive(Debug)]
pub(crate) struct Sources<'run, F, C, T> {
    wired: &'run Wired<'run, F, C, T>,
    staged: &'run Measured<Stage<'run>>,
    queue: Queue,
    located: OnceLock<Measured<Result<Described, Unlocated>>>,
    model: OnceLock<Option<Model>>,
}

impl<'run, F: Files, C: Cargo, T: Timer> Wired<'run, F, C, T> {
    pub(crate) fn new(files: &'run F, cargo: &'run C, timer: &'run T, shown: Timings) -> Wired<'run, F, C, T> {
        Wired { files, cargo, timer, total: Stopwatch::started(timer), shown }
    }

    pub(crate) fn files(&self) -> &'run F {
        self.files
    }

    pub(crate) fn timer(&self) -> &'run T {
        self.timer
    }

    pub(crate) fn table(&self, timings: impl Iterator<Item = Timing>) -> Table {
        Table::new(self.shown, timings.chain([self.total.stopped(Step::Total)]).collect())
    }
}

impl<'run, F: Files, C: Cargo, T: Timer> Sources<'run, F, C, T> {
    pub(crate) fn new(
        wired: &'run Wired<'run, F, C, T>,
        staged: &'run Measured<Stage<'run>>,
    ) -> Sources<'run, F, C, T> {
        Sources { wired, staged, queue: Queue::default(), located: OnceLock::new(), model: OnceLock::new() }
    }

    pub(crate) fn wired(&self) -> &'run Wired<'run, F, C, T> {
        self.wired
    }

    pub(crate) fn read(&self, worker: Worker) -> Measured<Readout> {
        Stopwatch::measured(self.wired.timer, Step::Read(worker), || {
            SourceFiles::read_staged(self.wired.files, self.staged.done())
        })
    }

    pub(crate) fn earlier(&self) -> impl Iterator<Item = Timing> {
        [self.staged.timing(), self.located().timing()].into_iter()
    }

    pub(crate) fn queue(&self) -> &Queue {
        &self.queue
    }

    pub(crate) fn located(&self) -> &Measured<Result<Described, Unlocated>> {
        self.located.get_or_init(|| {
            Stopwatch::measured(self.wired.timer, Step::Locate, || {
                CrateSources::read(self.wired.files, self.wired.cargo, &self.staged.done().package_directories())
            })
        })
    }

    pub(crate) fn model(&self, readout: &Readout) -> Option<&Model> {
        self.model
            .get_or_init(|| {
                self.located().done().as_ref().ok().map(|described| Reader::read(readout, described.crates()))
            })
            .as_ref()
    }
}
