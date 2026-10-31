use crate::escape::{EscapeKinds, Logged, RepeatKey};
use crate::event::{Event, Route};
use crate::head::Standing;
use crate::ids::{EscapeId, PrNumber, RunCount, Timestamp};
use crate::latch::Latch;
use crate::lease::LeaseSlot;
use crate::ledger::Ledger;
use crate::merges::{Reconciliation, Unreached};
use crate::predictions::{Measure, Observation, Predictions, Quantity};
use crate::reply::Accepted;
use crate::rework::PrRuns;
use crate::set_aside::{UnverifiedEscapes, one_line};
use crate::task::Progress;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

/// What `meta status` reads besides the log, each as read or why it could not be.
#[derive(Debug)]
pub struct Sources {
    pub github: Reconciliation,
    pub kinds: Result<EscapeKinds, Unreached>,
    pub predictions: Result<Predictions, Unreached>,
    /// CI runs of every PR in `Ledger::rework_prs`, for the report-only rework section.
    pub runs: PrRuns,
    pub unverified: Result<UnverifiedEscapes, Unreached>,
    pub standing: Standing,
}

/// Why P1's agent-start half cannot say "holds": meta sees a start without a lease only through an
/// `outside-start` escape, so a start nobody recorded is invisible to it.
const UNSEEN_STARTS: &str = "agent starts without a lease are not observable: meta sees only those an outside-start escape records, not the hook's denials";

/// What `meta status` tells the lead to do while the default data dir's log is unmarked.
const MARK_THE_LOG: &str = "run meta mark-real-log with a main build";

impl Ledger {
    /// A lease is flagged from its third resume onward.
    const FLAGGED_RESUMES: usize = 3;

    /// The report, and a `Falsified` event for each falsifying measure not latched before.
    pub fn status(&self, sources: Sources, at: Timestamp) -> Accepted {
        let (events, text) = self.report(sources, at);
        Accepted::new(events, text)
    }

    fn report(&self, sources: Sources, at: Timestamp) -> (Vec<Event>, String) {
        let Sources { github, kinds, predictions, runs: pr_runs, unverified, standing } = sources;
        let listed = |keep: fn(&Progress) -> bool| -> Vec<String> {
            self.tasks.values().filter(|task| keep(task)).map(|task| task.to_string()).collect()
        };
        let leases: Vec<String> = self
            .leases
            .iter()
            .filter_map(|(id, slot)| match slot {
                LeaseSlot::Unused(lease) if self.live(lease) => {
                    Some(format!("{id} for {}, {}", lease.task(), slot.deadline()))
                }
                _ => None,
            })
            .collect();
        let overdue: Vec<String> = self
            .leases
            .iter()
            .filter(|(_, slot)| {
                self.live(slot.lease())
                    && !self.waiting.contains_key(&slot.lease().task())
                    && !slot.is_finished()
                    && slot.deadline().passed(at)
            })
            .map(|(id, slot)| format!("{id} for {}: {}, {}", slot.lease().task(), slot.holder(), slot.deadline()))
            .collect();
        let main = kinds.as_ref().ok();
        let set_aside = unverified.as_ref().map_or_else(|_| Vec::new(), UnverifiedEscapes::logged);
        let open_count = self.open_escapes() + set_aside.len();
        let open_escapes: Vec<String> = (1..=self.escapes.len())
            .map(|number| EscapeId::after(number - 1))
            .filter_map(|id| {
                self.escape(id).filter(|logged| logged.is_open()).map(|logged| format!("{id} {}", logged.line(main)))
            })
            .chain(set_aside.iter().map(|logged| format!("unverified {}", one_line(&logged.line(main)))))
            .collect();
        let corrected: Vec<String> = (1..=self.escapes.len())
            .map(|number| EscapeId::after(number - 1))
            .filter_map(|id| self.escape(id).and_then(|logged| logged.correction(id)))
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
        let repeats = self.repeats(&set_aside, main);
        let most = repeats.iter().filter(|(_, count)| **count > 0).max_by_key(|(_, count)| **count);
        let most_line = most.map_or_else(|| "0".to_owned(), |(key, count)| format!("{count} ({key})"));
        let started = self.leases.values().filter(|slot| matches!(slot, LeaseSlot::Spent(_))).count();
        let starts_outside = self.escapes.iter().filter(|logged| logged.is_outside_start()).count();
        let merged = self.merged.len();
        let through_meta = self.merged.iter().filter(|(_, route)| *route == Route::Meta).count();
        let outside_logged = merged - through_meta;
        let runs = RunCount::total(self.merged.iter().map(|(runs, _)| *runs));
        let median = Quantity::median(self.merged.iter().map(|(runs, _)| runs.value()).collect());
        let median_text = median.map_or_else(|| "none yet".to_owned(), |median| median.to_string());
        let missing: Result<Vec<PrNumber>, String> = match &github {
            Reconciliation::Listed(merges) => {
                Ok(merges.prs().iter().filter(|pr| !self.landed.contains_key(pr)).copied().collect())
            }
            Reconciliation::Unavailable(why) => Err(format!("GitHub merged list unreadable: {why}")),
        };
        let observe = |measure: Measure| -> Observation {
            match measure {
                Measure::OpenEscapes => Ok(Quantity::count(open_count)),
                Measure::RepeatsOfOneKind => Ok(Quantity::count(most.map_or(0, |(_, count)| *count))),
                Measure::MedianCiRuns => median.ok_or_else(|| "no merged PRs yet".to_owned()),
                Measure::UnrecordedBypasses => {
                    let merges = self.unnamed_outside_merges(&kinds, &missing);
                    let starts = self.escapes.iter().filter(|logged| logged.is_outside_start()).count();
                    // A known bypass falsifies whatever else is unreadable; none known leaves the rest undecided.
                    match (merges, starts) {
                        (Ok(merges), starts) if merges + starts > 0 => Ok(Quantity::count(merges + starts)),
                        (Err(_), starts) if starts > 0 => Ok(Quantity::count(starts)),
                        (Err(why), _) => Err(why),
                        (Ok(_), _) => Err(UNSEEN_STARTS.to_owned()),
                    }
                }
            }
        };
        let (predicted, fresh) = match &predictions {
            Ok(predictions) => {
                let readings = predictions.read(observe);
                let mut latched = self.falsified.clone();
                let fresh: Vec<Event> = readings
                    .iter()
                    .flat_map(|reading| {
                        reading.falsifying.iter().map(move |(measure, observed)| (reading, *measure, observed))
                    })
                    .filter_map(|(reading, measure, observed)| match latched.entry((reading.id.clone(), measure)) {
                        Entry::Vacant(vacant) => {
                            vacant.insert(Latch::new(at, observed.clone()));
                            Some(Event::Falsified {
                                prediction: reading.id.clone(),
                                measure,
                                observed: observed.clone(),
                            })
                        }
                        Entry::Occupied(_) => None,
                    })
                    .collect();
                let lines = readings
                    .iter()
                    .map(|reading| {
                        let kept: String = latched
                            .iter()
                            .filter(|((prediction, _), _)| *prediction == reading.id)
                            .map(|(_, latch)| format!("; {}", latch.shown()))
                            .collect();
                        format!("{}{kept}", reading.line)
                    })
                    .collect();
                (lines, fresh)
            }
            Err(why) => (vec![format!("unavailable: {why}")], Vec::new()),
        };
        let side_note =
            unverified.map_or_else(|why| format!("unverified escapes unavailable: {why}\n"), |read| read.note());
        let text = [
            match standing {
                Standing::AwaitingMark => format!("warning: this log is not marked yet; {MARK_THE_LOG}\n"),
                Standing::Scratch | Standing::Real | Standing::MarkedElsewhere => String::new(),
            },
            section("open tasks", &listed(Progress::is_active)),
            section("deferred", &listed(|task| matches!(task, Progress::Deferred(_)))),
            section("live leases", &leases),
            section(&format!("open escapes: {open_count}"), &open_escapes),
            section("escapes corrected to historical", &corrected),
            section("latest review verdict per linked PR", &self.verdicts()),
            section(&format!("leases resumed {} or more times (flagged)", Self::FLAGGED_RESUMES), &resumes),
            section("overdue leases (flagged)", &overdue),
            section("waiting on the owner (flagged)", &self.waits(at)),
            section("idle (flagged)", &self.idle()),
            format!(
                "P1 agent starts through meta: {started}; prevented by the hook: {}; bypassed, from outside-start escapes: {starts_outside}; self-claims, not agent starts: {}\n",
                self.starts_denied, self.self_claims
            ),
            self.merges_line(through_meta, outside_logged, github),
            format!(
                "P2 open escapes: {open_count} ({} closed as historical); most repeats of one kind: {most_line}\n",
                self.closed_as_historical()
            ),
            side_note,
            format!("P6 CI runs per merged PR: {runs} runs over {merged} merged PRs; median {median_text}\n"),
            self.rework(&pr_runs),
            section("predictions", &predicted),
        ]
        .concat();
        (fresh, text)
    }

    /// PRs merged outside meta, logged or missing from the log, that no escape naming a merged PR names.
    fn unnamed_outside_merges(
        &self,
        kinds: &Result<EscapeKinds, Unreached>,
        missing: &Result<Vec<PrNumber>, String>,
    ) -> Result<usize, String> {
        let kinds = kinds.as_ref().map_err(|why| format!("escape kinds unreadable: {why}"))?;
        let named: BTreeSet<PrNumber> = self.escapes.iter().filter_map(|logged| logged.merged_pr(kinds)).collect();
        let missing = missing.as_ref().map_err(Clone::clone)?;
        let outside: BTreeSet<&PrNumber> = self.outside.iter().chain(missing).collect();
        Ok(outside.into_iter().filter(|pr| !named.contains(pr)).count())
    }

    fn open_escapes(&self) -> usize {
        self.escapes.iter().filter(|logged| logged.is_open()).count()
    }

    /// Escapes closed as historical, corrections included: shown beside P2, not judged by it.
    fn closed_as_historical(&self) -> usize {
        self.escapes.iter().filter(|logged| logged.is_historical()).count()
    }

    /// Repeats per kind, open or resolved, side-file escapes included: each kind's escapes after its
    /// first. Kinds that main does not list share one key.
    fn repeats<'a>(&'a self, set_aside: &'a [Logged], main: Option<&EscapeKinds>) -> BTreeMap<RepeatKey<'a>, usize> {
        let escapes = self.escapes.iter().chain(set_aside);
        let counts = escapes.fold(BTreeMap::<RepeatKey<'a>, usize>::new(), |mut counts, logged| {
            *counts.entry(logged.key(main)).or_default() += 1;
            counts
        });
        counts.into_iter().map(|(key, count)| (key, count - 1)).collect()
    }

    fn merges_line(&self, through_meta: usize, outside_logged: usize, github: Reconciliation) -> String {
        match github {
            Reconciliation::Listed(merges) => {
                let missing = merges.prs().iter().filter(|pr| !self.landed.contains_key(pr)).count();
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
