mod remembered;

use std::fmt;
use std::io;
use std::iter;
use std::num::NonZeroUsize;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, Scope, ScopedJoinHandle};

use std::collections::BTreeMap;

use algebra::{Monoid, Semigroup};
use lint_core::{Headline, Report, Violation};
use lint_sweep::Visited;
use newtype::Checked;
use serde::{Deserialize, Serialize};

pub(crate) use remembered::{Invocation, Kept, Store};

use super::stack_size::StackSize;
use super::timings::{Timing, Visiting};
use crate::cli::ignored::Ignored;
use invariants::{Ignoring, Rejoined};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    slot: Slot,
    text: String,
}

impl Rendered {
    pub(crate) fn dirty(slot: Slot, report: Report<'_>) -> Option<Rendered> {
        Some(report).filter(|found| found.dirty().is_some()).map(|found| Rendered { slot, text: found.to_string() })
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct FileOrder(usize);

impl From<usize> for FileOrder {
    fn from(index: usize) -> FileOrder {
        FileOrder(index)
    }
}

impl FileOrder {
    pub(crate) const LAST: FileOrder = FileOrder(usize::MAX);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FileLines {
    slot: Slot,
    order: FileOrder,
    text: WholeLines,
}

#[derive(newtype::Newtype, newtype::Arbitrary, newtype::Serialize, newtype::CheckedDeserialize, Clone, Debug)]
#[newtype(checked, strategy = "([^\\n]*\\n)+")]
struct WholeLines(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("filed text is one or more whole lines, each ending in a line break")]
struct Torn;

impl Checked for WholeLines {
    type Error = Torn;

    fn checked(text: &String) -> Result<&String, Torn> {
        text.chars().next_back().filter(|last| *last == Worked::LINE_END).map(|_| text).ok_or(Torn)
    }
}

impl AsRef<str> for WholeLines {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Filings(Vec<FileLines>);

#[derive(Debug, Default)]
pub(crate) struct Worked {
    rendered: Vec<Rendered>,
    filed: Vec<FileLines>,
    timings: Vec<Timing>,
    visited: Visiting,
}

impl Worked {
    const LINE_END: char = '\n';

    pub(crate) fn rendering(slot: Slot, reports: Vec<Report<'_>>) -> Worked {
        let rendered = reports.into_iter().filter_map(|report| Rendered::dirty(slot, report)).collect();
        Worked { rendered, ..Worked::default() }
    }

    pub(crate) fn filing(slot: Slot, order: FileOrder, violations: &[Violation<'_>]) -> Worked {
        let joined = violations.iter().map(ToString::to_string).fold(String::new(), |mut text, line| {
            text.push_str(&line);
            text.push(Self::LINE_END);
            text
        });
        let filed =
            WholeLines::try_new(joined).ignoring(Ignored::Unviolated).map(|text| FileLines { slot, order, text });
        Worked { filed: filed.into_iter().collect(), ..Worked::default() }
    }

    pub(crate) fn recalled(filings: Filings, order: FileOrder) -> Worked {
        let filed = filings.0.into_iter().map(|lines| FileLines { order, ..lines }).collect();
        Worked { filed, ..Worked::default() }
    }

    pub(crate) fn remembered(&self) -> Filings {
        Filings(self.filed.clone())
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
            slots.entry(part.slot).or_default().push_str(part.text.as_ref());
            slots
        });
        let assembled = grouped.into_iter().filter_map(|(slot, lines)| {
            headline(slot).map(|title| Rendered { slot, text: format!("{title}:\n{lines}") })
        });
        self.rendered.into_iter().chain(assembled).collect()
    }
}

impl Semigroup for Worked {
    fn combine(self, other: Worked) -> Worked {
        Worked {
            rendered: self.rendered.combine(other.rendered),
            filed: self.filed.combine(other.filed),
            timings: self.timings.combine(other.timings),
            visited: self.visited.combine(other.visited),
        }
    }
}

impl Monoid for Worked {}

impl FromIterator<Worked> for Worked {
    fn from_iter<Parts: IntoIterator<Item = Worked>>(parts: Parts) -> Worked {
        Monoid::concat(parts)
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

#[derive(Debug)]
pub(crate) struct Dumped<'model>(&'model graph::Model);

impl<'model> From<&'model graph::Model> for Dumped<'model> {
    fn from(model: &'model graph::Model) -> Dumped<'model> {
        Dumped(model)
    }
}
impl fmt::Display for Dumped<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&format!("{:#?}\n", self.0))
    }
}
