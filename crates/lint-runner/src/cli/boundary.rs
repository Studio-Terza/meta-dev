use std::fmt;
use std::io;
use std::iter;
use std::num::NonZeroUsize;
use std::ops::Range;
use std::panic::resume_unwind;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, Scope, ScopedJoinHandle};

use std::collections::BTreeMap;

use lint_core::{Headline, Report, Violation};

use super::stack_size::StackSize;
use super::timings::Timing;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Slot(usize);

impl Slot {
    pub(crate) const NONE: Slot = Slot(usize::MAX);

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FileOrder(usize);

impl From<usize> for FileOrder {
    fn from(index: usize) -> FileOrder {
        FileOrder(index)
    }
}

impl FileOrder {
    pub(crate) fn picked_from<Held>(self, held: &[Held]) -> Option<&Held> {
        held.get(self.0)
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
    rendered: Vec<Rendered>,
    filed: Vec<FileLines>,
    timings: Vec<Timing>,
    surveyed: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct Surveyed(Vec<String>);

impl Worked {
    const LINE_END: char = '\n';

    pub(crate) fn rendering(slot: Slot, reports: Vec<Report<'_>>) -> Worked {
        let rendered = reports.into_iter().filter_map(|report| Rendered::dirty(slot, report)).collect();
        Worked { rendered, ..Worked::default() }
    }

    pub(crate) fn filing(slot: Slot, order: FileOrder, violations: &[Violation<'_>]) -> Worked {
        let text = violations.iter().map(ToString::to_string).fold(String::new(), |mut text, line| {
            text.push_str(&line);
            text.push(Self::LINE_END);
            text
        });
        let filed = violations.first().map(|_| FileLines { slot, order, text });
        Worked { filed: filed.into_iter().collect(), ..Worked::default() }
    }

    pub(crate) fn surveying(survey: &impl fmt::Display) -> Worked {
        Worked { surveyed: vec![survey.to_string()], ..Worked::default() }
    }

    pub(crate) fn timed(timing: Timing) -> Worked {
        Worked { timings: vec![timing], ..Worked::default() }
    }

    pub(crate) fn timings(&self) -> &[Timing] {
        &self.timings
    }

    pub(crate) fn into_rendered(self, headline: impl Fn(Slot) -> Option<Headline>) -> Vec<Rendered> {
        let mut filed = self.filed;
        filed.sort_by_key(|part| (part.slot, part.order));
        let grouped = filed.into_iter().fold(BTreeMap::new(), |mut slots: BTreeMap<Slot, String>, part| {
            slots.entry(part.slot).or_default().push_str(&part.text);
            slots
        });
        let assembled = grouped.into_iter().filter_map(|(slot, lines)| {
            headline(slot).map(|title| Rendered { slot, text: format!("{title}:\n{lines}") })
        });
        self.rendered.into_iter().chain(assembled).collect()
    }
}

impl FromIterator<Worked> for Worked {
    fn from_iter<Parts: IntoIterator<Item = Worked>>(parts: Parts) -> Worked {
        parts.into_iter().fold(Worked::default(), |mut whole, part| {
            whole.rendered.extend(part.rendered);
            whole.filed.extend(part.filed);
            whole.timings.extend(part.timings);
            whole.surveyed.extend(part.surveyed);
            whole
        })
    }
}

impl From<&Worked> for Surveyed {
    fn from(worked: &Worked) -> Surveyed {
        Surveyed(worked.surveyed.clone())
    }
}

impl fmt::Display for Surveyed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|survey| formatter.write_str(survey))
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
