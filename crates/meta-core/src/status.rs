use crate::escape::{EscapeKinds, RepeatKey};
use crate::event::Route;
use crate::ids::{EscapeId, RunCount};
use crate::lease::LeaseSlot;
use crate::ledger::Ledger;
use crate::merges::{Reconciliation, Unreached};
use crate::predictions::{Measure, Observation, Predictions, Quantity};
use crate::reply::Accepted;
use crate::task::Progress;
use std::collections::BTreeMap;

/// What `meta status` reads besides the log, each as read or why it could not be.
#[derive(Debug)]
pub struct Sources {
    pub github: Reconciliation,
    pub kinds: Result<EscapeKinds, Unreached>,
    pub predictions: Result<Predictions, Unreached>,
}

impl Ledger {
    /// A lease is flagged from its third resume onward.
    const FLAGGED_RESUMES: usize = 3;

    pub fn status(&self, sources: Sources) -> Accepted {
        Accepted::new(Vec::new(), self.report(sources))
    }

    fn report(&self, sources: Sources) -> String {
        let Sources { github, kinds, predictions } = sources;
        let listed = |keep: fn(&Progress) -> bool| -> Vec<String> {
            self.tasks.values().filter(|task| keep(task)).map(|task| task.to_string()).collect()
        };
        let leases: Vec<String> = self
            .leases
            .iter()
            .filter_map(|(id, slot)| match slot {
                LeaseSlot::Unused(lease) if self.live(lease) => Some(format!("{id} for {}", lease.task())),
                _ => None,
            })
            .collect();
        let open_escapes: Vec<String> = (1..=self.escapes.len())
            .map(|number| EscapeId::after(number - 1))
            .filter_map(|id| self.escape(id).filter(|logged| logged.is_open()).map(|logged| logged.line(id)))
            .collect();
        let resumes: Vec<String> = self
            .leases
            .iter()
            .filter_map(|(id, slot)| match slot {
                LeaseSlot::Spent(spent) if spent.resumes() >= Self::FLAGGED_RESUMES => {
                    Some(format!("{id}: resumed {} times", spent.resumes()))
                }
                _ => None,
            })
            .collect();
        let repeats = self.repeats();
        let most = repeats.iter().filter(|(_, count)| **count > 0).max_by_key(|(_, count)| **count);
        let most_line = most.map_or_else(|| "0".to_owned(), |(key, count)| format!("{count} ({key})"));
        let started = self.leases.values().filter(|slot| matches!(slot, LeaseSlot::Spent(_))).count();
        let merged = self.merged.len();
        let through_meta = self.merged.iter().filter(|(_, route)| *route == Route::Meta).count();
        let outside_logged = merged - through_meta;
        let runs = RunCount::total(self.merged.iter().map(|(runs, _)| *runs));
        let median = Quantity::median(self.merged.iter().map(|(runs, _)| runs.value()).collect());
        let median_text = median.map_or_else(|| "none yet".to_owned(), |median| median.to_string());
        let missing = match &github {
            Reconciliation::Listed(merges) => Ok(merges.prs().iter().filter(|pr| !self.landed.contains(pr)).count()),
            Reconciliation::Unavailable(why) => Err(format!("GitHub merged list unreadable: {why}")),
        };
        let observe = |measure: Measure| -> Observation {
            match measure {
                Measure::OpenEscapes => Ok(Quantity::count(self.open_escapes())),
                Measure::RepeatsOfOneKind => Ok(Quantity::count(most.map_or(0, |(_, count)| *count))),
                Measure::MedianCiRuns => median.ok_or_else(|| "no merged PRs yet".to_owned()),
                Measure::UnrecordedBypasses => {
                    let kinds = kinds.as_ref().map_err(|why| format!("escape kinds unreadable: {why}"))?;
                    let recorded = self.escapes.iter().filter(|logged| kinds.bypass(logged.kind())).count();
                    let outside = outside_logged + missing.clone()?;
                    Ok(Quantity::count(outside.saturating_sub(recorded)))
                }
            }
        };
        let predicted = match &predictions {
            Ok(predictions) => predictions.lines(observe),
            Err(why) => vec![format!("unavailable: {why}")],
        };
        [
            section("open tasks", &listed(Progress::is_active)),
            section("deferred", &listed(|task| matches!(task, Progress::Deferred(_)))),
            section("live leases", &leases),
            section(&format!("open escapes: {}", self.open_escapes()), &open_escapes),
            section("latest review verdict per linked PR", &self.verdicts()),
            section(&format!("leases resumed {} or more times (flagged)", Self::FLAGGED_RESUMES), &resumes),
            format!(
                "P1 agent starts through meta: {started} of {}; self-claims, not agent starts: {}\n",
                started + self.starts_denied,
                self.self_claims
            ),
            self.merges_line(through_meta, outside_logged, github),
            format!("P2 open escapes: {}; most repeats of one kind: {most_line}\n", self.open_escapes()),
            format!("P6 CI runs per merged PR: {runs} runs over {merged} merged PRs; median {median_text}\n"),
            section("predictions", &predicted),
        ]
        .concat()
    }

    fn open_escapes(&self) -> usize {
        self.escapes.iter().filter(|logged| logged.is_open()).count()
    }

    /// Repeats per kind, open or resolved: each kind's escapes after its first.
    fn repeats(&self) -> BTreeMap<RepeatKey<'_>, usize> {
        let counts = self.escapes.iter().fold(BTreeMap::<RepeatKey<'_>, usize>::new(), |mut counts, logged| {
            *counts.entry(logged.key()).or_default() += 1;
            counts
        });
        counts.into_iter().map(|(key, count)| (key, count - 1)).collect()
    }

    fn merges_line(&self, through_meta: usize, outside_logged: usize, github: Reconciliation) -> String {
        match github {
            Reconciliation::Listed(merges) => {
                let missing = merges.prs().iter().filter(|pr| !self.landed.contains(pr)).count();
                format!(
                    "P1 merges through meta: {through_meta} of {}\nP1 merged outside meta: {} ({missing} missing from the log)\n",
                    through_meta + outside_logged + missing,
                    outside_logged + missing
                )
            }
            Reconciliation::Unavailable(why) => format!(
                "P1 merges: unavailable, GitHub merged list unreadable ({why}); logged: {through_meta} through meta, {outside_logged} outside\n"
            ),
        }
    }
}

fn section(heading: &str, lines: &[String]) -> String {
    let body = match lines.is_empty() {
        true => "  (none)\n".to_owned(),
        false => lines.iter().map(|line| format!("  {line}\n")).collect(),
    };
    format!("{heading}\n{body}")
}
