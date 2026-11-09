use std::fmt;
use std::io;
use std::iter;
use std::num::NonZeroUsize;
use std::ops::Range;
use std::panic::resume_unwind;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, Scope, ScopedJoinHandle};

use lint_core::Report;

use super::stack_size::StackSize;
use super::timings::Timing;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Slot(usize);

impl Slot {
    pub(crate) const NONE: Slot = Slot(usize::MAX);
}

impl From<usize> for Slot {
    fn from(index: usize) -> Slot {
        Slot(index)
    }
}

#[derive(Debug)]
pub(crate) struct Rendered {
    slot: Slot,
    text: String,
}

impl Rendered {
    pub(crate) fn dirty(slot: Slot, report: Report<'_>) -> Option<Rendered> {
        report.dirty().map(|found| Rendered { slot, text: found.to_string() })
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

#[derive(Debug, Default)]
pub(crate) struct Worked {
    rendered: Vec<Rendered>,
    timings: Vec<Timing>,
}

impl Worked {
    pub(crate) fn new(rendered: Vec<Rendered>, timings: Vec<Timing>) -> Worked {
        Worked { rendered, timings }
    }

    pub(crate) fn timed(timing: Timing) -> Worked {
        Worked { rendered: Vec::new(), timings: vec![timing] }
    }

    pub(crate) fn timings(&self) -> &[Timing] {
        &self.timings
    }

    pub(crate) fn into_rendered(self) -> Vec<Rendered> {
        self.rendered
    }
}

impl FromIterator<Worked> for Worked {
    fn from_iter<Parts: IntoIterator<Item = Worked>>(parts: Parts) -> Worked {
        parts.into_iter().fold(Worked::default(), |mut whole, part| {
            whole.rendered.extend(part.rendered);
            whole.timings.extend(part.timings);
            whole
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Queue {
    next: AtomicUsize,
}

impl Queue {
    pub(crate) fn claimed<'items, T>(&'items self, items: &'items [T]) -> impl Iterator<Item = &'items T> {
        iter::from_fn(move || items.get(self.next.fetch_add(1, Ordering::Relaxed)))
    }
}

#[derive(Debug)]
pub(crate) struct Locator<'scope>(Option<ScopedJoinHandle<'scope, ()>>);

impl<'scope> Locator<'scope> {
    pub(crate) fn spawned(scope: &'scope Scope<'scope, '_>, locate: impl FnOnce() + Send + 'scope) -> Locator<'scope> {
        Locator(thread::Builder::new().spawn_scoped(scope, locate).ok())
    }

    pub(crate) fn joined(self) {
        self.0.into_iter().for_each(|locating| locating.join().unwrap_or_else(|payload| resume_unwind(payload)));
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Workers(usize);

impl Workers {
    const MOST_READERS: usize = 4;

    pub(crate) fn available() -> Workers {
        Workers(thread::available_parallelism().map_or(1, NonZeroUsize::get).min(Self::MOST_READERS))
    }

    pub(crate) fn helpers(self) -> Range<usize> {
        1..self.0
    }
}

#[derive(Debug)]
pub(crate) struct Helper<'scope>(ScopedJoinHandle<'scope, Worked>);

pub(crate) type Helpers<'scope> = Vec<Helper<'scope>>;

impl<'scope> Helper<'scope> {
    pub(crate) fn spawned<'env>(
        scope: &'scope Scope<'scope, 'env>,
        work: impl FnOnce() -> Worked + Send + 'scope,
    ) -> io::Result<Helper<'scope>> {
        StackSize::LINT.builder().spawn_scoped(scope, work).map(Helper)
    }

    pub(crate) fn joined(self) -> Worked {
        self.0.join().unwrap_or_else(|payload| resume_unwind(payload))
    }
}
