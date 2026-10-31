use crate::event::Route;
use crate::ids::{Span, Timestamp};
use crate::ledger::Ledger;
use crate::predictions::Quantity;
use crate::task::Progress;

/// How many delivered tasks each side of the slope line takes. The brief for task-45 fixes it.
const WINDOW: usize = 10;

/// What the log's timestamps say about one task: when it opened, when it ended, how long it
/// waited on the owner, and how its own PR merged.
#[derive(Debug)]
pub(crate) struct Timeline {
    opened: Timestamp,
    ended: Option<Timestamp>,
    waited: Span,
    merged: Option<Route>,
}

impl Timeline {
    pub(crate) fn opened(at: Timestamp) -> Self {
        Self { opened: at, ended: None, waited: Span::of_seconds(0), merged: None }
    }

    /// The task was retired.
    pub(crate) fn end(&mut self, at: Timestamp) {
        self.ended = Some(at);
    }

    /// The task is done; `route` is how its own PR merged, none when another task's PR finished it.
    pub(crate) fn finish(&mut self, at: Timestamp, route: Option<Route>) {
        self.ended = Some(at);
        self.merged = route;
    }

    pub(crate) fn waited(&mut self, span: Span) {
        self.waited = self.waited.plus(span);
    }
}

/// Where a task ended up, or stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fate {
    MergedThroughMeta,
    MergedOutside,
    DoneByAnotherPr,
    Retired,
    Deferred,
    Open,
}

impl Fate {
    fn of(progress: &Progress, timeline: &Timeline) -> Self {
        match progress {
            Progress::Done(_) => match (progress.done_by(), timeline.merged) {
                (Some(_), _) => Self::DoneByAnotherPr,
                (None, Some(Route::Outside)) => Self::MergedOutside,
                (None, _) => Self::MergedThroughMeta,
            },
            Progress::Retired(_) => Self::Retired,
            Progress::Deferred(_) => Self::Deferred,
            Progress::Open(_) | Progress::Claimed(_) | Progress::Linked(_) => Self::Open,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::MergedThroughMeta => "merged through meta",
            Self::MergedOutside => "merged outside meta",
            Self::DoneByAnotherPr => "done by another PR",
            Self::Retired => "retired",
            Self::Deferred => "deferred",
            Self::Open => "open",
        }
    }
}

impl Ledger {
    /// The time and outcome section of `meta status`: report only, never read by a prediction.
    /// Everything comes from the log and `at`; GitHub is not asked.
    pub(crate) fn timing(&self, at: Timestamp) -> String {
        const HEADING: &str = "time and outcome per task (report only)\n";
        let lines: String = self
            .tasks
            .iter()
            .filter_map(|(task, progress)| self.timelines.get(task).map(|timeline| (task, progress, timeline)))
            .map(|(task, progress, timeline)| {
                let fate = Fate::of(progress, timeline);
                let (elapsed, label) = match timeline.ended {
                    Some(ended) => (timeline.opened.span_to(ended), "opened to done"),
                    None => (timeline.opened.span_to(at), "open so far"),
                };
                let ongoing =
                    self.waiting.get(task).map_or(Span::of_seconds(0), |waiting| waiting.started().span_to(at));
                let waited = timeline.waited.plus(ongoing);
                format!(
                    "  {task}: {}; {label} {} min; waiting on the owner {} min\n",
                    fate.name(),
                    elapsed.whole_minutes(),
                    waited.whole_minutes()
                )
            })
            .collect();
        let lines = match lines.is_empty() {
            true => "  (none)\n".to_owned(),
            false => lines,
        };
        format!("{HEADING}{lines}{}", self.slope())
    }

    /// Minutes to deliver and failed review rounds of each delivered task, in the order they were done.
    fn deliveries(&self) -> Vec<(u64, u64)> {
        self.delivered
            .iter()
            .filter_map(|task| {
                let timeline = self.timelines.get(task)?;
                let minutes = timeline.opened.span_to(timeline.ended?).whole_minutes();
                Some((minutes, self.failed_rounds(*task).value() as u64))
            })
            .collect()
    }

    /// The raw P4 signal: the most recent `WINDOW` delivered tasks against the `WINDOW` before them.
    fn slope(&self) -> String {
        let deliveries = self.deliveries();
        let split = deliveries.len().saturating_sub(WINDOW);
        let (before, recent) = deliveries.split_at(split);
        let before = &before[before.len().saturating_sub(WINDOW)..];
        let median = |rows: &[(u64, u64)], pick: fn(&(u64, u64)) -> u64| {
            Quantity::median(rows.iter().map(pick).collect()).map_or_else(|| "none yet".to_owned(), |m| m.to_string())
        };
        let minutes: fn(&(u64, u64)) -> u64 = |row| row.0;
        let rounds: fn(&(u64, u64)) -> u64 = |row| row.1;
        format!(
            "slope across delivered tasks, report only (P4 raw signal, no prediction reads it): {} delivered; \
             median minutes to deliver: most recent {} {}, the {} before {}; \
             median failed review rounds per task: most recent {}, the {} before {}\n",
            deliveries.len(),
            recent.len(),
            median(recent, minutes),
            before.len(),
            median(before, minutes),
            median(recent, rounds),
            before.len(),
            median(before, rounds),
        )
    }
}
