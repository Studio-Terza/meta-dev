use std::io;
use std::iter::Enumerate;
use std::iter::{Skip, StepBy};
use std::panic::resume_unwind;
use std::slice::Iter;
use std::thread::{Scope, ScopedJoinHandle};

use lint_layout::Failure;

use crate::cli::boundary::Rendered;
use crate::cli::stack_size::StackSize;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Shard {
    first: usize,
    width: usize,
}

impl Shard {
    pub(crate) fn lead(width: usize) -> Shard {
        Shard { first: 0, width }
    }

    pub(crate) fn new(first: usize, width: usize) -> Shard {
        Shard { first, width }
    }

    pub(crate) fn picks<T>(self, items: &[T]) -> StepBy<Skip<Enumerate<Iter<'_, T>>>> {
        items.iter().enumerate().skip(self.first).step_by(self.width)
    }
}

#[derive(Debug)]
pub(crate) struct Helper<'scope>(ScopedJoinHandle<'scope, Result<Vec<Rendered>, Failure>>);

pub(crate) type Helpers<'scope> = Vec<Result<Helper<'scope>, Shard>>;

impl<'scope> Helper<'scope> {
    pub(crate) fn spawned<'env>(
        scope: &'scope Scope<'scope, 'env>,
        work: impl FnOnce() -> Result<Vec<Rendered>, Failure> + Send + 'scope,
    ) -> io::Result<Helper<'scope>> {
        StackSize::LINT.builder().spawn_scoped(scope, work).map(Helper)
    }

    pub(crate) fn joined(self, failed_slot: usize) -> Vec<Rendered> {
        self.0
            .join()
            .unwrap_or_else(|payload| resume_unwind(payload))
            .unwrap_or_else(|failure| vec![Rendered::failed(failed_slot, &failure)])
    }
}
