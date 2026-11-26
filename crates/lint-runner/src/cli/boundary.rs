use std::fmt;
use std::io;
use std::num::NonZeroUsize;
use std::ops::Range;
use std::thread::{self, Scope, ScopedJoinHandle};

use std::collections::BTreeMap;

use lint_core::{Headline, Violation};
use lint_sweep::Visited;

use super::stack_size::StackSize;
use super::timings::{Timing, Visiting};
use crate::cli::ignored::Ignored;
use invariants::{Ignoring, Rejoined};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Slot(usize);

impl Slot {
    pub(crate) fn picked<Item>(self, items: &[Item]) -> Option<&Item> {
        items.get(self.0)
    }
}

impl From<usize> for Slot {
    fn from(index: usize) -> Slot {
        Slot(index)
    }
}

#[derive(Debug)]
pub(crate) struct Rendered {
    text: String,
}

impl fmt::Display for Rendered {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FileOrder(usize);

impl From<usize> for FileOrder {
    fn from(index: usize) -> FileOrder {
        FileOrder(index)
    }
}

#[derive(Debug)]
struct FileLines {
    slot: Slot,
    order: FileOrder,
    text: String,
}

#[derive(Debug, Default)]
pub(crate) struct Worked {
    filed: Vec<FileLines>,
    timings: Vec<Timing>,
    visited: Visiting,
}

impl Worked {
    const LINE_END: char = '\n';

    pub(crate) fn filing(slot: Slot, order: FileOrder, violations: &[Violation<'_>]) -> Worked {
        let text = violations.iter().map(ToString::to_string).fold(String::new(), |mut text, line| {
            text.push_str(&line);
            text.push(Self::LINE_END);
            text
        });
        let filed = violations.first().map(|_| FileLines { slot, order, text });
        Worked { filed: filed.into_iter().collect(), ..Worked::default() }
    }

    pub(crate) fn timed(timing: Timing) -> Worked {
        Worked { timings: vec![timing], ..Worked::default() }
    }

    pub(crate) fn timings(&self) -> &[Timing] {
        &self.timings
    }

    pub(crate) fn visiting(visited: Visited) -> Worked {
        Worked { visited: Visiting::from(visited), ..Worked::default() }
    }

    pub(crate) fn visited(&self) -> &Visiting {
        &self.visited
    }

    pub(crate) fn into_rendered(self, headline: impl Fn(Slot) -> Option<Headline>) -> Vec<Rendered> {
        let mut filed = self.filed;
        filed.sort_by_key(|part| (part.slot, part.order));
        let grouped = filed.into_iter().fold(BTreeMap::new(), |mut slots: BTreeMap<Slot, String>, part| {
            slots.entry(part.slot).or_default().push_str(&part.text);
            slots
        });
        grouped
            .into_iter()
            .filter_map(|(slot, lines)| headline(slot).map(|title| Rendered { text: format!("{title}:\n{lines}") }))
            .collect()
    }
}

impl FromIterator<Worked> for Worked {
    fn from_iter<Parts: IntoIterator<Item = Worked>>(parts: Parts) -> Worked {
        parts.into_iter().fold(Worked::default(), |mut all, part| {
            all.filed.extend(part.filed);
            all.timings.extend(part.timings);
            Worked { visited: all.visited.combine(part.visited), ..all }
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Workers(usize);

impl Workers {
    const MOST_READERS: usize = 4;

    pub(crate) fn available() -> Workers {
        Workers(
            thread::available_parallelism()
                .ignoring(Ignored::Unknown)
                .map_or(1, NonZeroUsize::get)
                .min(Self::MOST_READERS),
        )
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
        self.0.join().rejoined()
    }
}
