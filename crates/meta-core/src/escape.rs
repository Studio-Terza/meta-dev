use crate::event::Event;
use crate::ids::Reason;
use crate::ledger::Ledger;
use crate::reply::Accepted;

impl Ledger {
    pub fn escape(&self, reason: Reason) -> Accepted {
        let open = self.escapes.len() + 1;
        let repeats = self.escapes.iter().filter(|earlier| **earlier == reason).count();
        let text = format!("escape recorded: {reason}\nopen escapes: {open}\nthis reason repeated: {repeats}\n");
        Accepted { events: vec![Event::Escaped { reason }], text }
    }
}
