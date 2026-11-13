mod contexts;
mod gathering;

pub(crate) use contexts::{Gleans, Holding, Priming, Settled};
pub(crate) use gathering::{Claimed, Gathering};

use std::sync::OnceLock;

use cargo_ports::Cargo;
use file_ports::Files;
use graph::Model;
use graph_reader::{FileFacts, Reader};
use lint_core::{FileHead, Readout};
use lint_reader::{CrateSources, Described, Partition, SharedRead, Stage, Unlocated};

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
    shared: &'run SharedRead<'run>,
    queue: Queue,
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

    pub(crate) fn table(&self, timings: impl Iterator<Item = Timing>) -> Table {
        Table::new(self.shown, timings.chain([self.total.stopped(Step::Total)]).collect())
    }
}

impl<'run, F: Files, C: Cargo, T: Timer> Sources<'run, F, C, T> {
    pub(crate) fn new(
        wired: &'run Wired<'run, F, C, T>,
        staged: &'run Measured<Stage<'run>>,
        shared: &'run SharedRead<'run>,
        heads: &'run OnceLock<Vec<FileHead>>,
    ) -> Sources<'run, F, C, T> {
        Sources {
            wired,
            staged,
            shared,
            queue: Queue::default(),
            located: OnceLock::new(),
            gathering: Gathering::new(heads),
        }
    }

    pub(crate) fn wired(&self) -> &'run Wired<'run, F, C, T> {
        self.wired
    }

    pub(crate) fn read(&self, worker: Worker) -> Measured<Readout> {
        Stopwatch::measured(self.wired.timer, Step::Read(worker), || {
            self.shared.read(self.wired.files, Partition::from(worker))
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

    pub(crate) fn gathered<'read>(&self, readout: &'read Readout, worker: Worker) -> Lowering<'read> {
        let stopwatch = Stopwatch::started(self.wired.timer);
        let share = self.gathering.gathered(readout, self.shared.heads());
        let lowered = stopwatch.stopped(Step::Lower(worker));
        let claimed = share.claimed().to_vec();
        let merged = share.into_merging().map(|complete| {
            let merging = Stopwatch::started(self.wired.timer);
            self.gathering.settled(complete, |facts| self.merged(facts));
            merging.stopped(Step::Model)
        });
        Lowering { timings: [lowered].into_iter().chain(merged).collect(), claimed }
    }

    fn merged(&self, facts: Vec<FileFacts>) -> Option<Model> {
        let model = FileFacts::merged(facts);
        self.located().done().as_ref().ok().map(|described| Reader::rooted(model, described.crates()))
    }

    pub(crate) fn gathering(&self) -> &Gathering<'run> {
        &self.gathering
    }
}

#[derive(Debug)]
pub(crate) struct Lowering<'read> {
    timings: Vec<Timing>,
    claimed: Vec<Claimed<'read>>,
}

impl<'read> Lowering<'read> {
    pub(crate) fn timings(&self) -> &[Timing] {
        &self.timings
    }

    pub(crate) fn claimed(&self) -> &[Claimed<'read>] {
        &self.claimed
    }
}
