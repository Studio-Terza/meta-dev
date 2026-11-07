use std::fmt;
use std::io;
use std::iter;
use std::num::NonZeroUsize;
use std::ops::Range;
use std::panic::resume_unwind;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, Scope, ScopedJoinHandle};

use code_model::Model;
use code_reader::Reader;
use lint_core::{Readout, Report};

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

#[derive(Debug, Default)]
pub(crate) struct Queue {
    next: AtomicUsize,
    model: OnceLock<Model>,
}

impl Queue {
    pub(crate) fn claimed<'items, T>(&'items self, items: &'items [T]) -> impl Iterator<Item = &'items T> {
        iter::from_fn(move || items.get(self.next.fetch_add(1, Ordering::Relaxed)))
    }

    pub(crate) fn model(&self, readout: &Readout) -> &Model {
        self.model.get_or_init(|| Reader::read(readout))
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
pub(crate) struct Helper<'scope>(ScopedJoinHandle<'scope, Vec<Rendered>>);

pub(crate) type Helpers<'scope> = Vec<Helper<'scope>>;

impl<'scope> Helper<'scope> {
    pub(crate) fn spawned<'env>(
        scope: &'scope Scope<'scope, 'env>,
        work: impl FnOnce() -> Vec<Rendered> + Send + 'scope,
    ) -> io::Result<Helper<'scope>> {
        StackSize::LINT.builder().spawn_scoped(scope, work).map(Helper)
    }

    pub(crate) fn joined(self) -> Vec<Rendered> {
        self.0.join().unwrap_or_else(|payload| resume_unwind(payload))
    }
}
