use std::fmt;
use std::io;
use std::iter::Enumerate;
use std::iter::{Skip, StepBy};
use std::num::NonZeroUsize;
use std::panic::resume_unwind;
use std::slice::Iter;
use std::thread::{self, Scope, ScopedJoinHandle};

use lint_core::Report;
use lint_layout::Failure;

use super::stack_size::StackSize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Slot {
    Check(usize),
    Query(usize),
    Failed,
}

#[derive(Debug)]
pub(crate) struct Rendered {
    slot: Slot,
    text: String,
}

impl Rendered {
    pub(crate) fn render(slot: Slot, report: &Report<'_>) -> Rendered {
        Rendered { slot, text: report.to_string() }
    }

    pub(crate) fn failed(failure: &Failure) -> Rendered {
        Rendered { slot: Slot::Failed, text: format!("{failure}\n") }
    }

    pub(crate) fn slot(&self) -> Slot {
        self.slot
    }
}

impl fmt::Display for Rendered {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Shard {
    first: usize,
    width: usize,
}

impl Shard {
    pub(crate) fn lead(width: usize) -> Shard {
        Shard { first: 0, width }
    }

    fn spawned<'scope>(
        self,
        scope: &'scope Scope<'scope, '_>,
        work: &'scope (impl Fn(Shard) -> Vec<Rendered> + Sync),
    ) -> ScopedJoinHandle<'scope, Vec<Rendered>> {
        scope.spawn(move || work(self))
    }

    pub(crate) fn picks<T>(self, items: &[T]) -> StepBy<Skip<Enumerate<Iter<'_, T>>>> {
        items.iter().enumerate().skip(self.first).step_by(self.width)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Workers(usize);

impl Workers {
    const MOST_READERS: usize = 4;

    pub(crate) fn available() -> Workers {
        Workers(thread::available_parallelism().map_or(1, NonZeroUsize::get).min(Self::MOST_READERS))
    }

    pub(crate) fn shards(self) -> impl Iterator<Item = Shard> {
        (0..self.0).map(move |first| Shard { first, width: self.0 })
    }

    pub(crate) fn spread(self, work: impl Fn(Shard) -> Vec<Rendered> + Sync) -> Vec<Rendered> {
        thread::scope(|scope| Self::joined(self.shards().map(|shard| shard.spawned(scope, &work)).collect()))
    }

    fn joined(workers: Vec<ScopedJoinHandle<'_, Vec<Rendered>>>) -> Vec<Rendered> {
        workers.into_iter().flat_map(|worker| worker.join().unwrap_or_else(|payload| resume_unwind(payload))).collect()
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

    pub(crate) fn joined(self) -> Vec<Rendered> {
        self.0
            .join()
            .unwrap_or_else(|payload| resume_unwind(payload))
            .unwrap_or_else(|failure| vec![Rendered::failed(&failure)])
    }
}
