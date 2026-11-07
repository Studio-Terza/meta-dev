use super::{Deadline, Timeout, Wall};

impl Deadline {
    #[must_use]
    pub fn after(start: Wall, timeout: Timeout) -> Deadline {
        Deadline(Wall(start.0.saturating_add(i64::from(timeout.0))))
    }
}
