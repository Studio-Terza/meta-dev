use crate::event::{Attempt, Event, Record};
use crate::ids::{LeaseId, PrNumber, Reason, RunCount, TaskId};
use crate::lease::{Lease, LeaseSlot};
use crate::task::{Progress, Task};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Default)]
pub struct Ledger {
    pub(crate) tasks: BTreeMap<TaskId, Progress>,
    pub(crate) links: BTreeMap<PrNumber, TaskId>,
    pub(crate) leases: BTreeMap<LeaseId, LeaseSlot>,
    pub(crate) escapes: Vec<Reason>,
    pub(crate) merged: Vec<RunCount>,
    pub(crate) denied: Vec<Attempt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Corrupt {
    record: usize,
    why: String,
}

impl fmt::Display for Corrupt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "log record {} does not fit the log before it: {}", self.record, self.why)
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
            Event::Linked { task, pr, pin } => {
                let linked = match self.take(*task)? {
                    Progress::Claimed(claimed) => Ok(claimed.link(*pr, pin.clone())),
                    Progress::Linked(linked) => {
                        self.links.remove(&linked.pr());
                        Ok(linked.relink(*pr, pin.clone()))
                    }
                    other => Err(format!("{task} linked while {other}")),
                }?;
                self.links.insert(*pr, *task);
                self.tasks.insert(*task, Progress::Linked(linked));
            }
            Event::Merged { pr, runs, .. } => {
                self.links.get(pr).ok_or_else(|| format!("unlinked {pr} merged"))?;
                self.merged.push(*runs);
            }
            Event::Done { task } => {
                let done = match self.take(*task)? {
                    Progress::Linked(linked) => {
                        self.links.remove(&linked.pr());
                        Ok(linked.finish())
                    }
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

    pub(crate) fn live(&self, lease: &Lease) -> bool {
        self.tasks.get(&lease.task()).is_some_and(|task| !task.is_done())
    }
}
