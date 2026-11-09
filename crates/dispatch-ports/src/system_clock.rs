use std::time::{Duration, SystemTime};

use port_core::Pass;

use crate::{Bell, ClockFiller, Pause, Wall, clock_ops};

#[derive(Debug)]
pub struct SystemClock {
    bell: Bell,
}

impl SystemClock {
    #[must_use]
    pub fn new(bell: &Bell) -> SystemClock {
        SystemClock { bell: bell.clone() }
    }
}

impl ClockFiller for SystemClock {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        Wall::from(SystemTime::now())
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause) -> Wall {
        self.bell.waited(Duration::from(pause));
        Wall::from(SystemTime::now())
    }
}
