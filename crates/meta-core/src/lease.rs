use crate::event::Event;
use crate::ids::{LeaseId, TaskId};

#[derive(Debug)]
pub struct Lease {
    id: LeaseId,
    task: TaskId,
}

#[derive(Debug)]
pub struct Spent;

impl Lease {
    pub(crate) fn issue(id: LeaseId, task: TaskId) -> Self {
        Self { id, task }
    }

    pub(crate) fn task(&self) -> TaskId {
        self.task
    }

    pub(crate) fn spend(self) -> (Spent, Event) {
        (Spent, Event::LeaseUsed { lease: self.id })
    }
}

#[derive(Debug)]
pub enum LeaseSlot {
    Unused(Lease),
    Spent(Spent),
}
