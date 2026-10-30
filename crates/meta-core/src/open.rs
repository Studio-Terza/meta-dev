use crate::event::Event;
use crate::ids::{TaskId, Title};
use crate::ledger::Ledger;
use crate::reply::Accepted;

impl Ledger {
    pub fn open(&self, title: Title) -> Accepted {
        let task = TaskId::after(self.tasks.len());
        Accepted { events: vec![Event::Opened { task, title }], text: format!("{task}\n") }
    }
}
