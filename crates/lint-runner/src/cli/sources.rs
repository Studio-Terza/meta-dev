use cargo_ports::Cargo;
use file_ports::Files;
use lint_layout::Crates;
use lint_reader::{CrateSources, Stage, Unlocated};

use super::boundary::Queue;

#[derive(Debug)]
pub(crate) struct Ports<'run, F, C> {
    files: &'run F,
    cargo: &'run C,
}

#[derive(Debug)]
pub(crate) struct Sources<'run, F, C> {
    ports: Ports<'run, F, C>,
    stage: &'run Stage<'run>,
    queue: Queue,
}

impl<'run, F: Files, C: Cargo> Ports<'run, F, C> {
    pub(crate) fn new(files: &'run F, cargo: &'run C) -> Ports<'run, F, C> {
        Ports { files, cargo }
    }

    pub(crate) fn files(&self) -> &'run F {
        self.files
    }

    pub(crate) fn located(&self, stage: &Stage<'_>) -> Result<Crates, Unlocated> {
        CrateSources::read(self.files, self.cargo, &stage.package_directories())
    }
}

impl<'run, F, C> Sources<'run, F, C> {
    pub(crate) fn new(ports: Ports<'run, F, C>, stage: &'run Stage<'run>) -> Sources<'run, F, C> {
        Sources { ports, stage, queue: Queue::default() }
    }

    pub(crate) fn ports(&self) -> &Ports<'run, F, C> {
        &self.ports
    }

    pub(crate) fn stage(&self) -> &'run Stage<'run> {
        self.stage
    }

    pub(crate) fn queue(&self) -> &Queue {
        &self.queue
    }
}
