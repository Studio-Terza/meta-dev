use std::fmt;
use std::iter::Enumerate;
use std::iter::{Skip, StepBy};
use std::num::NonZeroUsize;
use std::panic::resume_unwind;
use std::slice::Iter;
use std::thread::{self, Scope, ScopedJoinHandle};

use lint_core::Report;

use super::stack_size::StackSize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Slot {
    Check(usize),
    Query(usize),
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
    fn spawned<'scope>(
        self,
        scope: &'scope Scope<'scope, '_>,
        work: &'scope (impl Fn(Shard) -> Vec<Rendered> + Sync),
    ) -> Spread<'scope> {
        match StackSize::LINT.builder().spawn_scoped(scope, move || work(self)) {
            Ok(handle) => Spread::Spawned(handle),
            Err(_) => Spread::Inline(work(self)),
        }
    }

    pub(crate) fn picks<T>(self, items: &[T]) -> StepBy<Skip<Enumerate<Iter<'_, T>>>> {
        items.iter().enumerate().skip(self.first).step_by(self.width)
    }
}

#[derive(Debug)]
enum Spread<'scope> {
    Spawned(ScopedJoinHandle<'scope, Vec<Rendered>>),
    Inline(Vec<Rendered>),
}

impl Spread<'_> {
    fn rendered(self) -> Vec<Rendered> {
        match self {
            Spread::Spawned(handle) => handle.join().unwrap_or_else(|payload| resume_unwind(payload)),
            Spread::Inline(rendered) => rendered,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Workers(usize);

impl Workers {
    const MOST_READERS: usize = 4;

    pub(crate) fn available() -> Workers {
        Workers(thread::available_parallelism().map_or(1, NonZeroUsize::get).min(Self::MOST_READERS))
    }

    pub(crate) fn at_most(self, most: usize) -> Workers {
        Workers(self.0.min(most).max(1))
    }

    pub(crate) fn shards(self) -> impl Iterator<Item = Shard> {
        (0..self.0).map(move |first| Shard { first, width: self.0 })
    }

    pub(crate) fn spread(self, work: impl Fn(Shard) -> Vec<Rendered> + Sync) -> Vec<Rendered> {
        thread::scope(|scope| {
            let spread: Vec<Spread<'_>> = self.shards().map(|shard| shard.spawned(scope, &work)).collect();
            spread.into_iter().flat_map(Spread::rendered).collect()
        })
    }
}
