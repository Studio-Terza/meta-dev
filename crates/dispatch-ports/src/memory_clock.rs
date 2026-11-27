use std::sync::Mutex;
use std::time::Duration;

use invariants::Unpoisoned;
use port_core::Pass;

use crate::{Bell, ClockFiller, Moment, Pause, Wall, Woke, clock_ops};

#[derive(Debug)]
pub struct MemoryClock {
    now: Mutex<Wall>,
}

impl MemoryClock {
    #[must_use]
    pub fn starting(now: Wall) -> MemoryClock {
        MemoryClock { now: Mutex::new(now) }
    }
}

impl ClockFiller for MemoryClock {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        *self.now.lock().unpoisoned()
    }

    fn moment(&self, _pass: Pass<'_, clock_ops::Moment>) -> Moment {
        Moment::from(*self.now.lock().unpoisoned())
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause, bell: &Bell) -> Wall {
        let mut now = self.now.lock().unpoisoned();
        *now = match bell.waited(Duration::ZERO) {
            Woke::Lapsed => now.after(pause),
            Woke::Rung => *now,
        };
        *now
    }
}
