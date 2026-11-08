use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{Deadline, Pause, Timeout, Wall};

impl Deadline {
    #[must_use]
    pub fn after(start: Wall, timeout: Timeout) -> Deadline {
        Deadline(Wall(start.0.saturating_add(i64::from(timeout.0))))
    }
}

impl Wall {
    #[must_use]
    pub fn after(self, pause: Pause) -> Wall {
        Wall(self.0.saturating_add(i64::from(pause.0)))
    }
}

impl From<SystemTime> for Wall {
    fn from(time: SystemTime) -> Wall {
        let seconds = time.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        Wall(i64::try_from(seconds).unwrap_or(i64::MAX))
    }
}

impl From<Pause> for Duration {
    fn from(pause: Pause) -> Duration {
        Duration::from_secs(u64::from(pause.0))
    }
}
