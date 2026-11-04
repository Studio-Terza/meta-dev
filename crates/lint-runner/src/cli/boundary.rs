use std::fmt;
use std::io;
use std::iter::{self, RepeatN};
use std::num::NonZeroUsize;
use std::panic::resume_unwind;
use std::thread::{self, Scope, ScopedJoinHandle};

use lint_core::Report;

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
pub(crate) struct Workers(usize);

impl Workers {
    const MOST_READERS: usize = 4;

    pub(crate) fn available() -> Workers {
        Workers(thread::available_parallelism().map_or(1, NonZeroUsize::get).min(Self::MOST_READERS))
    }

    pub(crate) fn helpers(self) -> RepeatN<()> {
        iter::repeat_n((), self.0.saturating_sub(1))
    }
}

#[derive(Debug)]
pub(crate) struct Helper<'scope>(ScopedJoinHandle<'scope, Vec<Rendered>>);

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
