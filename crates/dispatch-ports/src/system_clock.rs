use std::thread;
use std::time::{Duration, SystemTime};

use newtype::Wrapping;
use port_core::{Pass, Real};

use crate::{ClockFiller, Pause, Wall, clock_ops};

#[derive(Debug)]
pub struct SystemClock;

impl Real for SystemClock {}

impl ClockFiller for SystemClock {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        Wall::from(SystemTime::now())
    }

    fn paused(&self, _pass: Pass<'_, clock_ops::Paused>, pause: Pause) -> Wall {
        thread::sleep(Duration::from_secs(u64::from(pause.into_inner())));
        Wall::from(SystemTime::now())
    }
}
