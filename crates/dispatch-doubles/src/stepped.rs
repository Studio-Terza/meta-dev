use std::sync::Mutex;

use dispatch_ports::{ClockFiller, Pause, Wall, clock_ops};
use invariants::Unpoisoned;
use port_core::Pass;

#[derive(Debug)]
pub struct Stepped {
    now: Mutex<Wall>,
}

impl Stepped {
    #[must_use]
    pub fn starting(start: Wall) -> Stepped {
        Stepped { now: Mutex::new(start) }
    }
}

impl ClockFiller for Stepped {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        *self.now.lock().unpoisoned()
    }

    fn paused(&self, _pass: Pass<'_, clock_ops::Paused>, pause: Pause) -> Wall {
        let mut now = self.now.lock().unpoisoned();
        *now = Stepped::after(*now, pause);
        *now
    }
}
