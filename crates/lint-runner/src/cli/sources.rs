use lint_reader::Stage;

use super::boundary::Queue;

#[derive(Debug)]
pub(crate) struct Sources<'run, F> {
    files: &'run F,
    stage: &'run Stage<'run>,
    queue: Queue,
}

impl<'run, F> Sources<'run, F> {
    pub(crate) fn new(files: &'run F, stage: &'run Stage<'run>) -> Self {
        Sources { files, stage, queue: Queue::default() }
    }

    pub(crate) const fn files(&self) -> &'run F {
        self.files
    }

    pub(crate) const fn stage(&self) -> &'run Stage<'run> {
        self.stage
    }

    pub(crate) const fn queue(&self) -> &Queue {
        &self.queue
    }
}
