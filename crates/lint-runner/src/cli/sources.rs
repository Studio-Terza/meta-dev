use std::mem;
use std::sync::{Mutex, OnceLock, PoisonError};

use cargo_ports::Cargo;
use code_model::Model;
use code_reader::{FileFacts, ListedFile, Listing, Reader};
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
    gathering: Gathering,
}

#[derive(Debug, Default)]
struct Gathering {
    claims: Queue,
    deposited: Mutex<Vec<FileFacts>>,
    model: OnceLock<Option<Model>>,
}

impl Gathering {
    fn deposited(&self, facts: Vec<FileFacts>, listing: &Listing<'_>) -> Option<Vec<FileFacts>> {
        let mut held = self.deposited.lock().unwrap_or_else(PoisonError::into_inner);
        held.extend(facts);
        Some(held.len()).filter(|count| *count == listing.entries().len()).map(|_| mem::take(&mut *held))
    }
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
        Sources { wired, staged, queue: Queue::default(), located: OnceLock::new(), gathering: Gathering::default() }
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

    pub(crate) fn lowered(&self, readout: &Readout, worker: Worker) -> Vec<Timing> {
        let listing = Listing::of_files(readout.files());
        let lowering = Stopwatch::started(self.wired.timer);
        let facts: Vec<FileFacts> = self.gathering.claims.claimed(listing.entries()).map(ListedFile::lowered).collect();
        let lowered = lowering.stopped(Step::Lower(worker));
        let merged = self.gathering.deposited(facts, &listing).map(|complete| {
            let merging = Stopwatch::started(self.wired.timer);
            self.gathering.model.get_or_init(|| self.merged(complete));
            merging.stopped(Step::Model)
        });
        [lowered].into_iter().chain(merged).collect()
    }

    fn merged(&self, facts: Vec<FileFacts>) -> Option<Model> {
        let model = FileFacts::merged(facts);
        self.located().done().as_ref().ok().map(|described| Reader::rooted(model, described.crates()))
    }

    pub(crate) fn model(&self) -> Option<&Model> {
        self.gathering.model.wait().as_ref()
    }
}
