use std::time::{Duration, SystemTime};

use crate::dispatch::{Bell, ClockFiller, Moment, Pause, Wall, clock_ops};
use crate::port::Pass;

#[derive(Debug)]
pub struct SystemClock;

impl ClockFiller for SystemClock {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        Wall::from(SystemTime::now())
    }

    fn moment(&self, _pass: Pass<'_, clock_ops::Moment>) -> Moment {
        Moment::from(SystemTime::now())
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause, bell: &Bell) -> Wall {
        let _woke = bell.waited(Duration::from(pause));
        Wall::from(SystemTime::now())
    }
}
