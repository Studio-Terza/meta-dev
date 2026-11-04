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

#[derive(Debug)]
pub(crate) struct Rendered {
    slot: usize,
    text: String,
}

impl Rendered {
    pub(crate) fn render(slot: usize, report: &Report<'_>) -> Rendered {
        Rendered { slot, text: report.to_string() }
    }

    pub(crate) fn failed(slot: usize, failure: &Failure) -> Rendered {
        Rendered { slot, text: format!("{failure}\n") }
    }

    pub(crate) fn slot(&self) -> usize {
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

    pub(crate) fn helper_shards(self) -> impl Iterator<Item = Shard> {
        (1..self.0).map(move |first| Shard { first, width: self.0 })
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
