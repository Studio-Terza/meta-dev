use std::thread;
use std::time::{Duration, SystemTime};

use port_core::Pass;

use crate::{ClockFiller, Pause, Wall, clock_ops};

#[derive(Debug)]
pub struct SystemClock;

impl ClockFiller for SystemClock {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        Wall::from(SystemTime::now())
    }

    fn paused(&self, _pass: Pass<'_, clock_ops::Paused>, pause: Pause) -> Wall {
        thread::sleep(Duration::from(pause));
        Wall::from(SystemTime::now())
    }
}
