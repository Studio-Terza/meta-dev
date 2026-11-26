mod contexts;
mod gathering;

pub(crate) use contexts::{Holding, Settled};
use gathering::Merging;
pub(crate) use gathering::{Claimed, Gathering};

use std::sync::OnceLock;

use cargo_ports::Cargo;
use file_ports::Files;
use lint_core::{FileHead, Readout};
use lint_reader::{CrateSources, Crew, Described, Partition, Reads, SharedRead, Stage, Unlocated};
use lint_sweep::Gathers;

use timer_ports::Timer;

use super::timings::{Measured, Step, Stopwatch, Table, Timing, Timings, Visiting, Worker};

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
    shared: &'run SharedRead<'run>,
    located: OnceLock<Measured<Result<Described, Unlocated>>>,
    gathering: Gathering<'run>,
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

    pub(crate) fn table(&self, timings: impl Iterator<Item = Timing>, visited: Visiting) -> Table {
        Table::new(self.shown, timings.chain([self.total.stopped(Step::Total)]).collect(), visited)
    }
}

impl<'run, F: Files, C: Cargo, T: Timer> Sources<'run, F, C, T> {
    pub(crate) fn new(
        wired: &'run Wired<'run, F, C, T>,
        staged: &'run Measured<Stage<'run>>,
        shared: &'run SharedRead<'run>,
        heads: &'run OnceLock<Vec<FileHead>>,
    ) -> Sources<'run, F, C, T> {
        Sources { wired, staged, shared, located: OnceLock::new(), gathering: Gathering::new(heads) }
    }

    pub(crate) fn wired(&self) -> &'run Wired<'run, F, C, T> {
        self.wired
    }

    pub(crate) fn read(&self, worker: Worker, reader: impl Reads) -> Measured<Readout> {
        Stopwatch::measured(self.wired.timer, Step::Read(worker), || reader.read(self.wired.files))
    }

    pub(crate) fn crew(&self, helpers: impl IntoIterator<Item = Worker>) -> Crew<'run, 'run> {
        self.shared.crew(helpers.into_iter().map(Partition::from))
    }

    pub(crate) fn earlier(&self) -> impl Iterator<Item = Timing> {
        [self.staged.timing()].into_iter()
    }

    pub(crate) fn located(&self) -> &Measured<Result<Described, Unlocated>> {
        self.located.get_or_init(|| {
            Stopwatch::measured(self.wired.timer, Step::Locate, || {
                CrateSources::read(self.wired.files, self.wired.cargo, &self.staged.done().package_directories())
            })
        })
    }

    pub(crate) fn gathered<'read, Part, Filed: Gathers<Part>>(
        &self,
        readout: &'read Readout,
        worker: Worker,
        starting: impl Fn(Claimed<'read>) -> Filed,
    ) -> Lowering<Part>
    where
        T: Sync,
    {
        let stopwatch = Stopwatch::started(self.wired.timer);
        let share = self.gathering.gathered(readout, self.shared.heads(), starting);
        let lowered = stopwatch.stopped(Step::Lower(worker));
        share.lowered(lowered, |complete, lowering| self.settled(complete, lowering))
    }

    fn settled<Part>(&self, complete: Merging<'run>, lowering: Lowering<Part>) -> Lowering<Part> {
        let timer = self.wired.timer;
        complete.settled(&self.gathering, lowering, |parts| {
            Stopwatch::measured(timer, Step::Contexts, || parts.into_iter().collect())
        })
    }

    pub(crate) fn gathering(&self) -> &Gathering<'run> {
        &self.gathering
    }
}

#[derive(Debug)]
pub(crate) struct Lowering<Part> {
    timings: Vec<Timing>,
    filed: Vec<Part>,
}

impl<Part> Lowering<Part> {
    pub(crate) fn timings(&self) -> &[Timing] {
        &self.timings
    }

    pub(crate) fn filed(&mut self) -> Vec<Part> {
        std::mem::take(&mut self.filed)
    }

    fn timed(self, timing: Timing) -> Lowering<Part> {
        Lowering { timings: self.timings.into_iter().chain([timing]).collect(), ..self }
    }
}
