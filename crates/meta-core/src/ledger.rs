use crate::escape::Logged;
use crate::event::{Attempt, Event, Record, Route};
use crate::ids::{EscapeId, LeaseId, PrNumber, PredictionId, RunCount, TaskId, Timestamp};
use crate::latch::Latch;
use crate::lease::{Deadline, Lease, LeaseSlot, Purpose};
use crate::predictions::Measure;
use crate::task::{Linked, Progress, Task};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct Ledger {
    pub(crate) tasks: BTreeMap<TaskId, Progress>,
    pub(crate) leases: BTreeMap<LeaseId, LeaseSlot>,
    pub(crate) escapes: Vec<Logged>,
    pub(crate) merged: Vec<(RunCount, Route)>,
    pub(crate) landed: BTreeSet<PrNumber>,
    /// PRs logged as merged outside meta.
    pub(crate) outside: BTreeSet<PrNumber>,
    pub(crate) starts_denied: usize,
    pub(crate) self_claims: usize,
    /// Each prediction's falsifying measures, latched at their first time.
    pub(crate) falsified: BTreeMap<(PredictionId, Measure), Latch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Corrupt {
    record: usize,
    why: String,
}

impl Corrupt {
    pub fn in_log(&self, place: &str) -> String {
        format!(
            "{place} record {} does not fit the log before it: {}. Next: meta escape <kind> <reason>, then repair {place}.",
            self.record, self.why
        )
    }
}

impl Ledger {
    pub fn fold(records: &[Record]) -> Result<Self, Corrupt> {
        records.iter().enumerate().try_fold(Self::default(), |ledger, (index, record)| {
            ledger.apply(&record.event, record.at).map_err(|why| Corrupt { record: index + 1, why })
        })
    }

    fn apply(mut self, event: &Event, at: Timestamp) -> Result<Self, String> {
        match event {
            Event::Opened { task, title } => {
                let fresh = Progress::Open(Task::new(*task, title.clone()));
                self.tasks.insert(*task, fresh).map_or(Ok(()), |_| Err(format!("{task} opened twice")))?;
            }
            Event::Claimed { task, agent } => {
                let claimed = match self.take(*task)? {
                    Progress::Open(open) => Ok(Progress::Claimed(open.claim(agent.clone()))),
                    other => Err(format!("{task} claimed while {other}")),
                }?;
                self.tasks.insert(*task, claimed);
                self.self_claims += usize::from(agent.is_lead());
            }
            Event::Leased { lease, task, deadline } => {
                self.tasks.get(task).ok_or_else(|| format!("{lease} leased for unknown {task}"))?;
                self.issue(Lease::issue(*lease, *task, Purpose::Work, at, Deadline::of(*deadline)))?;
            }
            Event::ReviewLeased { lease, task, pr, head, deadline } => {
                match self.tasks.get(task) {
                    Some(Progress::Linked(linked)) if linked.pr() == *pr => Ok(()),
                    _ => Err(format!("{lease} leased to review {pr}, which {task} does not link")),
                }?;
                let purpose = Purpose::Review { pr: *pr, head: head.clone() };
                self.issue(Lease::issue(*lease, *task, purpose, at, Deadline::of(*deadline)))?;
            }
            Event::Reviewed { lease, verdict } => match self.leases.get_mut(lease) {
                Some(LeaseSlot::Spent(spent)) => spent.judge(*verdict)?,
                _ => Err(format!("{lease} reviewed but not used"))?,
            },
            Event::Started { lease, agent, deadline } => match self.leases.get_mut(lease) {
                Some(LeaseSlot::Spent(spent)) => spent.start(agent.clone(), Deadline::of(*deadline))?,
                _ => Err(format!("{lease} started an agent but is not used"))?,
            },
            Event::Resumed { lease } => match self.leases.get_mut(lease) {
                Some(LeaseSlot::Spent(spent)) => spent.resume(),
                _ => Err(format!("{lease} resumed but not used"))?,
            },
            Event::Retired { task, reason } => {
                let retired = match self.take(*task)? {
                    Progress::Open(open) => Ok(open.retire(reason.clone())),
                    Progress::Claimed(claimed) => Ok(claimed.retire(reason.clone())),
                    Progress::Linked(linked) => Ok(linked.retire(reason.clone())),
                    other => Err(format!("{task} retired while {other}")),
                }?;
                self.tasks.insert(*task, Progress::Retired(retired));
            }
            Event::Deferred { task, reason } => {
                let deferred = match self.take(*task)? {
                    Progress::Open(open) => Ok(open.defer(reason.clone())),
                    other => Err(format!("{task} deferred while {other}")),
                }?;
                self.tasks.insert(*task, Progress::Deferred(deferred));
            }
            Event::Undeferred { task } => {
                let open = match self.take(*task)? {
                    Progress::Deferred(deferred) => Ok(deferred.undefer()),
                    other => Err(format!("{task} undeferred while {other}")),
                }?;
                self.tasks.insert(*task, Progress::Open(open));
            }
            Event::Linked { task, pr, pin, .. } => {
                let linked = match self.take(*task)? {
                    Progress::Claimed(claimed) => Ok(claimed.link(*pr, pin.clone())),
                    Progress::Linked(linked) => Ok(linked.relink(*pr, pin.clone())),
                    other => Err(format!("{task} linked while {other}")),
                }?;
                self.tasks.insert(*task, Progress::Linked(linked));
            }
            Event::Merged { pr, runs, route, .. } => {
                self.linked(*pr).ok_or_else(|| format!("unlinked {pr} merged"))?;
                self.landed.insert(*pr);
                if *route == Route::Outside {
                    self.outside.insert(*pr);
                }
                self.merged.push((*runs, *route));
            }
            Event::Done { task } => {
                let done = match self.take(*task)? {
                    Progress::Linked(linked) => Ok(linked.finish()),
                    other => Err(format!("{task} done while {other}")),
                }?;
                self.tasks.insert(*task, Progress::Done(done));
            }
            Event::Escaped { kind, reason, pr, checked, build } => {
                self.escapes.push(Logged::new(kind.clone(), reason.clone(), *pr, *checked, *build));
            }
            Event::Resolved { escape, task } => {
                match self.tasks.get(task) {
                    Some(Progress::Done(_)) => Ok(()),
                    Some(other) => Err(format!("{escape} resolved against {other}, which is not done")),
                    None => Err(format!("{escape} resolved against unknown {task}")),
                }?;
                let logged = self.escape_mut(*escape).ok_or_else(|| format!("unknown {escape} resolved"))?;
                logged.resolve(*escape, *task)?;
            }
            Event::ResolvedHistorical { escape, reason } => {
                let logged = self.escape_mut(*escape).ok_or_else(|| format!("unknown {escape} resolved"))?;
                logged.resolve_historical(*escape, reason.clone())?;
            }
            Event::Corrected { escape, reason } => {
                let logged = self.escape_mut(*escape).ok_or_else(|| format!("unknown {escape} corrected"))?;
                logged.correct(*escape, reason.clone())?;
            }
            Event::LeaseUsed { lease } => {
                let spent = match self.leases.remove(lease) {
                    Some(LeaseSlot::Unused(unused)) => Ok(unused.spend().0),
                    _ => Err(format!("{lease} used but not unused")),
                }?;
                self.leases.insert(*lease, LeaseSlot::Spent(spent));
            }
            Event::Falsified { prediction, measure, observed } => {
                self.falsified
                    .entry((prediction.clone(), *measure))
                    .or_insert_with(|| Latch::new(at, observed.clone()));
            }
            Event::Denied { attempt: Attempt::AgentStart } => self.starts_denied += 1,
            Event::Denied { attempt: Attempt::Merge } | Event::RealLog => {}
        }
        Ok(self)
    }

    fn issue(&mut self, lease: Lease) -> Result<(), String> {
        let id = lease.id();
        self.leases.insert(id, LeaseSlot::Unused(lease)).map_or(Ok(()), |_| Err(format!("{id} issued twice")))
    }

    fn escape_mut(&mut self, escape: EscapeId) -> Option<&mut Logged> {
        escape.index().and_then(|index| self.escapes.get_mut(index))
    }

    pub(crate) fn escape(&self, escape: EscapeId) -> Option<&Logged> {
        escape.index().and_then(|index| self.escapes.get(index))
    }

    fn take(&mut self, task: TaskId) -> Result<Progress, String> {
        self.tasks.remove(&task).ok_or_else(|| format!("{task} is unknown"))
    }

    pub(crate) fn linked(&self, pr: PrNumber) -> Option<&Task<Linked>> {
        self.tasks.values().find_map(|task| match task {
            Progress::Linked(linked) if linked.pr() == pr => Some(linked),
            _ => None,
        })
    }

    pub(crate) fn live(&self, lease: &Lease) -> bool {
        self.tasks.get(&lease.task()).is_some_and(Progress::is_active)
    }
}
