use crate::event::{Attempt, Event, Record, Route};
use crate::ids::{LeaseId, PrNumber, Reason, RunCount, TaskId};
use crate::lease::{Lease, LeaseSlot};
use crate::task::{Linked, Progress, Task};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Ledger {
    pub(crate) tasks: BTreeMap<TaskId, Progress>,
    pub(crate) leases: BTreeMap<LeaseId, LeaseSlot>,
    pub(crate) escapes: Vec<Reason>,
    pub(crate) merged: Vec<(RunCount, Route)>,
    pub(crate) denied: Vec<Attempt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Corrupt {
    record: usize,
    why: String,
}

impl Corrupt {
    pub fn in_log(&self, place: &str) -> String {
        format!(
            "{place} record {} does not fit the log before it: {}. Next: meta escape <reason>, then repair {place}.",
            self.record, self.why
        )
    }
}

impl Ledger {
    pub fn fold(records: &[Record]) -> Result<Self, Corrupt> {
        records.iter().enumerate().try_fold(Self::default(), |ledger, (index, record)| {
            ledger.apply(&record.event).map_err(|why| Corrupt { record: index + 1, why })
        })
    }

    fn apply(mut self, event: &Event) -> Result<Self, String> {
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
            }
            Event::Leased { lease, task } => {
                self.tasks.get(task).ok_or_else(|| format!("{lease} leased for unknown {task}"))?;
                let slot = LeaseSlot::Unused(Lease::issue(*lease, *task));
                self.leases.insert(*lease, slot).map_or(Ok(()), |_| Err(format!("{lease} issued twice")))?;
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
                self.merged.push((*runs, *route));
            }
            Event::Done { task } => {
                let done = match self.take(*task)? {
                    Progress::Linked(linked) => Ok(linked.finish()),
                    other => Err(format!("{task} done while {other}")),
                }?;
                self.tasks.insert(*task, Progress::Done(done));
            }
            Event::Escaped { reason } => self.escapes.push(reason.clone()),
            Event::LeaseUsed { lease } => {
                let spent = match self.leases.remove(lease) {
                    Some(LeaseSlot::Unused(unused)) => Ok(unused.spend().0),
                    _ => Err(format!("{lease} used but not unused")),
                }?;
                self.leases.insert(*lease, LeaseSlot::Spent(spent));
            }
            Event::Denied { attempt } => self.denied.push(*attempt),
        }
        Ok(self)
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
        self.tasks.get(&lease.task()).is_some_and(|task| !task.is_done())
    }
}
