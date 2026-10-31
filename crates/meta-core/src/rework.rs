use crate::ids::{FailedRounds, PrNumber, RunCount, TaskId, Verdict};
use crate::lease::{LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::merges::Unreached;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// CI runs of each PR in the rework report, each as read through the GitHub port or why it could not be.
pub type PrRuns = BTreeMap<PrNumber, Result<RunCount, Unreached>>;

/// What one task's PRs cost: the failed review rounds, and the CI runs when GitHub could be read.
struct Row {
    task: TaskId,
    stage: &'static str,
    prs: Vec<PrNumber>,
    failed: FailedRounds,
    runs: Result<RunCount, Unreached>,
}

impl Row {
    /// Rework is any failed review round, or a CI run beyond the first, or runs that could not be read.
    fn has_rework(&self) -> bool {
        self.failed.value() > 0 || self.runs.as_ref().map_or(true, |runs| runs.value() > 1)
    }

    /// Worst first: more failed rounds, then more CI runs.
    fn badness(&self) -> (FailedRounds, u64) {
        (self.failed, self.runs.as_ref().map_or(0, RunCount::value))
    }

    fn measures(&self) -> String {
        match &self.runs {
            Ok(runs) => format!("failed review rounds {}, CI runs {runs}", self.failed),
            Err(_) => format!("failed review rounds {}, CI runs unavailable", self.failed),
        }
    }
}

impl fmt::Display for Row {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prs: Vec<String> = self.prs.iter().map(ToString::to_string).collect();
        write!(formatter, "{} ({}, {}): {}", self.task, prs.join(", "), self.stage, self.measures())?;
        match &self.runs {
            Ok(_) => Ok(()),
            Err(why) => write!(formatter, " ({why})"),
        }
    }
}

impl Ledger {
    /// Every PR any task was linked to that has not merged: the PRs whose CI runs the rework report
    /// reads from GitHub. A merged PR's runs were recorded when it merged.
    pub fn rework_prs(&self) -> BTreeSet<PrNumber> {
        self.linked_prs.values().flatten().filter(|pr| !self.landed.contains_key(pr)).copied().collect()
    }

    pub(crate) fn failed_rounds(&self, task: TaskId) -> FailedRounds {
        let failed = self.leases.values().filter(|slot| match slot {
            LeaseSlot::Spent(spent) => {
                spent.lease().task() == task
                    && matches!(spent.lease().purpose(), Purpose::Review { .. })
                    && spent.verdict() == Some(Verdict::Fail)
            }
            LeaseSlot::Unused(_) | LeaseSlot::Replaced(_) => false,
        });
        FailedRounds::of(failed.count())
    }

    fn rework_rows(&self, runs: &PrRuns) -> Vec<Row> {
        let unread = || Unreached::by(&"not read from GitHub");
        self.linked_prs
            .iter()
            .filter_map(|(task, prs)| {
                let progress = self.tasks.get(task)?;
                let read: Result<Vec<RunCount>, Unreached> = prs
                    .iter()
                    .map(|pr| match self.landed.get(pr) {
                        Some(recorded) => Ok(*recorded),
                        None => runs.get(pr).cloned().unwrap_or_else(|| Err(unread())),
                    })
                    .collect();
                Some(Row {
                    task: *task,
                    stage: progress.stage(),
                    prs: prs.iter().copied().collect(),
                    failed: self.failed_rounds(*task),
                    runs: read.map(|read| RunCount::total(read.into_iter())),
                })
            })
            .collect()
    }

    /// The rework section of `meta status`: report only, never read by a prediction.
    pub(crate) fn rework(&self, runs: &PrRuns) -> String {
        const HEADING: &str = "rework per task (report only; not part of P6)\n";
        let rows = self.rework_rows(runs);
        let listed: Vec<&Row> = rows.iter().filter(|row| row.has_rework()).collect();
        let lines = match listed.is_empty() {
            true => "  (none)\n".to_owned(),
            false => listed.iter().map(|row| format!("  {row}\n")).collect(),
        };
        let repeated = rows.iter().filter(|row| row.failed.value() >= 2).count();
        let worst = listed
            .iter()
            .max_by_key(|row| row.badness())
            .map_or_else(|| "none".to_owned(), |row| format!("{} ({})", row.task, row.measures()));
        format!(
            "{HEADING}{lines}rework total: tasks with 2 or more failed review rounds: {repeated} of {} tasks with a PR; worst: {worst}\n",
            rows.len()
        )
    }
}
